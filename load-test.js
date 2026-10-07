// k6 load test, run by ./performance_test.sh (docker-compose-performance.yml).
//
// Built on the shared OpenAPI coverage module (mairie360/CICD `tests/k6/coverage.js`, MAIR-194):
// every operation of the spec served by the API needs exactly one handler ("METHOD /path", path
// as in openapi.json), k6 aborts at init otherwise. When you add an endpoint, add its handler to
// `readHandlers` (GET) or `writeHandlers` (any other method) and send its request through
// `request()` (raw `http.*` calls are not counted).
//
// High load on a volume seed (MAIR-474): the performance stack also runs init-perf.sql (300
// formations of 10 modules, 2 000 learners enrolled in 15 formations each, 150 000 completed
// modules). Three scenarios:
// - `reads`: the GET operations, ramping up to the read VUs of the profile (PROFILES). The admin lists read a random page (their
//   OFFSET cost grows with it), the learner routes run as a random seeded learner on one of their
//   formations, with a token signed here like the Admin's;
// - `writes`: every other operation with the write VUs of the profile, each handler undoing what it did when the API
//   allows it. Seeded learners complete module 6 of their formations (modules 1 to 5 are done),
//   an idempotent upsert that locks their enrolment row;
// - `formations_rush`: `GET /formations/` as learners at the fixed arrival rate of the profile, failing if k6 has
//   to drop iterations (the API no longer keeps up).
//
// The API cannot create formations, modules or attachments: they come from init-test.sql
// (formation 1000, the enrolment writes) and init-perf.sql (the formations the reads use).
//
// Unenrolling an agent who is not enrolled answers 404, so the write VUs must never share an
// agent: each one enrolls and unenrolls its own seeded learner in formation 1000, picked by its VU
// id (unique across the scenarios, fewer than the 2 000 learners).
import crypto from 'k6/crypto';
import encoding from 'k6/encoding';
import http from 'k6/http';
import exec from 'k6/execution';
import { check, fail, sleep } from 'k6';
import { createCoverage, loadSpec } from '/coverage.js';

const BASE_URL = (__ENV.BASE_URL || 'http://localhost:3006').replace(/\/+$/, '');

// HS256 admin JWT (sub=1, the Admin seeded by liquibase), signed at init with the stack's
// JWT_SECRET and valid for 2 hours, like .zap/scan.sh and the Postman collection do. No token is
// committed (MAIR-428): the repository is public. `JWT` still overrides it.
function signJwt(sub, role, secret, ttlSeconds) {
  const segment = (value) => encoding.b64encode(JSON.stringify(value), 'rawurl');
  const header = segment({ alg: 'HS256', typ: 'JWT' });
  const payload = segment({ sub: String(sub), role, exp: Math.floor(Date.now() / 1000) + ttlSeconds });
  const signature = crypto.hmac('sha256', secret, `${header}.${payload}`, 'base64rawurl');
  return `${header}.${payload}.${signature}`;
}

if (!__ENV.JWT && !__ENV.JWT_SECRET) {
  throw new Error('Set JWT_SECRET (the API\'s secret) or JWT (a ready-made admin token)');
}
const TOKEN = __ENV.JWT || signJwt(1, 'admin', __ENV.JWT_SECRET, 2 * 3600);
const AUTH = { Authorization: `Bearer ${TOKEN}` };

// Rows of init-perf.sql.
const LEARNERS = { first: 200001, count: 2000 };
const PERF_FORMATIONS = { first: 5000, count: 300 };
const ENROLMENTS_PER_LEARNER = 15;
const MODULES_PER_FORMATION = 10;
const COMPLETED_MODULES = 5;
// Admin list pages (`limit` default 50) and their sizes, fixtures included.
const ADMIN_PAGE = 50;
const CATALOGUE_SIZE = 300;
const LEARNERS_SIZE = 2000;

// Load profile (MAIR-474), K6_PROFILE:
// - `ci` (default): what the CI runner holds with the same strict thresholds. The runner
//   (ubuntu-latest, 4 vCPU) hosts the API, Postgres, Redis and k6 together;
// - `stress`: the high load, run by hand (`K6_PROFILE=stress ./performance_test.sh`) to find
//   the breaking point on a larger machine, not on every push.
const PROFILES = {
  ci: { readVus: 30, writeVus: 4, rushRate: 30 },
  stress: { readVus: 100, writeVus: 10, rushRate: 100 },
};
const PROFILE = PROFILES[__ENV.K6_PROFILE || 'ci'];
if (!PROFILE) throw new Error(`Unknown K6_PROFILE ${__ENV.K6_PROFILE}: ${Object.keys(PROFILES).join(', ')}`);

// Fixed-rate `GET /formations/` as learners.
const FORMATIONS_RUSH_RATE = PROFILE.rushRate; // requests per second
const FORMATIONS_RUSH_BUDGET_MS = 200;

const randomInt = (max) => Math.floor(Math.random() * max);
const randomPage = (size) => randomInt(Math.ceil(size / ADMIN_PAGE)) * ADMIN_PAGE;

const learnerTokens = {};

/** A random seeded learner: its id, `Authorization` header and one of its formations. */
function randomLearner() {
  const rank = randomInt(LEARNERS.count);
  const id = LEARNERS.first + rank;
  if (!learnerTokens[id]) {
    learnerTokens[id] = { Authorization: `Bearer ${signJwt(id, 'user', __ENV.JWT_SECRET, 2 * 3600)}` };
  }
  const formationRank = (rank + 20 * randomInt(ENROLMENTS_PER_LEARNER)) % PERF_FORMATIONS.count;
  return {
    id,
    headers: learnerTokens[id],
    formationId: PERF_FORMATIONS.first + formationRank,
    moduleId: (rank) => 10000 + MODULES_PER_FORMATION * formationRank + rank,
  };
}

// Seeded learner of the write VU, enrolled and unenrolled in formation 1000 (init-test.sql),
// which no learner follows otherwise.
const agentId = () => LEARNERS.first + exec.vu.idInTest - 1;
const FORMATION_ID = 1000;

// p(95) latency budget of each family of operations, in ms (reference machine).
const READ_BUDGET_MS = 200;
const WRITE_BUDGET_MS = 500;

const READ_METHODS = ['get', 'head', 'options'];

/** The served spec restricted to the operations whose method passes `keep`. */
function specSubset(spec, keep) {
  const paths = {};
  for (const path of Object.keys(spec.paths)) {
    const kept = {};
    for (const method of Object.keys(spec.paths[path])) {
      if (keep(method)) kept[method] = spec.paths[path][method];
    }
    if (Object.keys(kept).length > 0) paths[path] = kept;
  }
  return Object.assign({}, spec, { paths });
}

/**
 * Raw call for the fixtures of setup(), teardown() and the write handlers, outside the coverage
 * count. Tagged `op: fixture` so it stays out of the per-operation latency thresholds, but it
 * still counts in `http_req_failed`. Aborts the handler (or setup) on a non-2xx answer.
 */
function fixture(method, path, body) {
  const res = http.request(
    method,
    `${BASE_URL}${path}`,
    body === undefined ? null : JSON.stringify(body),
    { headers: Object.assign({ 'Content-Type': 'application/json' }, AUTH), tags: { op: 'fixture' } },
  );
  if (res.status < 200 || res.status >= 300) {
    fail(`fixture ${method} ${path} answered ${res.status}: ${res.body}`);
  }
  return res;
}

function enroll(userId, formationId) {
  fixture('POST', `/api/v1/admin/formations/${formationId}/`, { user_id: userId });
}

function unenroll(userId, formationId) {
  fixture('DELETE', `/api/v1/admin/users/${userId}/${formationId}/`);
}

const spec = loadSpec();

const readHandlers = {
  'GET /health': ({ request }) => check(request(), { 'health 200': (r) => r.status === 200 }),
  'GET /ready': ({ request }) => check(request(), { 'ready 200': (r) => r.status === 200 }),

  // Management view.
  'GET /api/v1/admin/formations/': ({ request }) =>
    check(request({ query: { details: true, offset: randomPage(CATALOGUE_SIZE) } }), {
      'catalog 200': (r) => r.status === 200,
      'catalog reads the seed': (r) => r.status === 200 && r.json('formations').length > 0,
    }),
  'GET /api/v1/admin/formations/{formation_id}/': ({ request }) =>
    check(request({ path: { formation_id: randomLearner().formationId }, query: { details: true } }), {
      'catalog formation 200': (r) => r.status === 200,
      'catalog formation reads its modules': (r) => r.status === 200 && r.json('modules').length === MODULES_PER_FORMATION,
    }),
  'GET /api/v1/admin/users/': ({ request }) =>
    check(request({ query: { offset: randomPage(LEARNERS_SIZE) } }), {
      'learners 200': (r) => r.status === 200,
      'learners reads the seed': (r) => r.status === 200 && r.json('users').length > 0,
    }),
  'GET /api/v1/admin/users/{user_id}/': ({ request }) =>
    check(request({ path: { user_id: randomLearner().id }, query: { details: true } }), {
      'learner 200': (r) => r.status === 200,
      'learner reads the enrolments': (r) => r.status === 200 && r.json('formations').length >= ENROLMENTS_PER_LEARNER,
    }),
  'GET /api/v1/admin/users/{user_id}/{formation_id}/': ({ request }) => {
    const learner = randomLearner();
    check(
      request({ path: { user_id: learner.id, formation_id: learner.formationId }, query: { details: true } }),
      {
        'learner formation 200': (r) => r.status === 200,
        'learner formation reads its modules': (r) => r.status === 200 && r.json('modules').length === MODULES_PER_FORMATION,
      },
    );
  },

  // View of a seeded learner, on one of their formations.
  'GET /api/v1/formations/': ({ request }) =>
    check(request({ headers: randomLearner().headers }), {
      'my formations 200': (r) => r.status === 200,
      'my formations reads the enrolments': (r) => r.status === 200 && r.json('formations').length >= ENROLMENTS_PER_LEARNER,
    }),
  'GET /api/v1/formations/{formation_id}/': ({ request }) => {
    const learner = randomLearner();
    check(request({ path: { formation_id: learner.formationId }, headers: learner.headers }), {
      'my formation 200': (r) => r.status === 200,
      'my formation reads its modules': (r) => r.status === 200 && r.json('modules').length === MODULES_PER_FORMATION,
    });
  },
  'GET /api/v1/formations/{formation_id}/{module_id}/': ({ request }) => {
    const learner = randomLearner();
    check(
      request({
        path: { formation_id: learner.formationId, module_id: learner.moduleId(randomInt(MODULES_PER_FORMATION)) },
        headers: learner.headers,
      }),
      {
        'my module 200': (r) => r.status === 200,
        'my module reads its attachments': (r) => r.status === 200 && r.json('files').length === 2,
      },
    );
  },
  'GET /api/v1/formations/{formation_id}/{module_id}/{attachment_id}/': ({ request }) => {
    const learner = randomLearner();
    const moduleId = learner.moduleId(randomInt(MODULES_PER_FORMATION));
    check(
      request({
        path: {
          formation_id: learner.formationId,
          module_id: moduleId,
          attachment_id: 20000 + 2 * (moduleId - 10000) + randomInt(2),
        },
        headers: learner.headers,
      }),
      {
        'attachment url 200': (r) => r.status === 200,
        'attachment url is signed': (r) => r.status === 200 && typeof r.json('url') === 'string',
      },
    );
  },
};

const writeHandlers = {

  // Enrollment: enroll → unenroll, each VU on its own agent (see the header).
  'POST /api/v1/admin/formations/{formation_id}/': ({ request }) => {
    check(request({ path: { formation_id: FORMATION_ID }, body: { user_id: agentId() } }), {
      'enroll 200': (r) => r.status === 200,
    });
    unenroll(agentId(), FORMATION_ID);
  },
  'DELETE /api/v1/admin/users/{user_id}/{formation_id}/': ({ request }) => {
    enroll(agentId(), FORMATION_ID);
    check(request({ path: { user_id: agentId(), formation_id: FORMATION_ID } }), {
      'unenroll 204': (r) => r.status === 204,
    });
  },

  // Progress of a seeded learner: the module after the ones init-perf.sql completed (an
  // idempotent upsert, no inverse route).
  'PATCH /api/v1/formations/{formation_id}/{module_id}/': ({ request }) => {
    const learner = randomLearner();
    check(
      request({
        path: { formation_id: learner.formationId, module_id: learner.moduleId(COMPLETED_MODULES) },
        headers: learner.headers,
      }),
      { 'complete module 200': (r) => r.status === 200 },
    );
  },
};

const reads = createCoverage(readHandlers, {
  spec: specSubset(spec, (method) => READ_METHODS.includes(method)),
});
const writes = createCoverage(writeHandlers, {
  spec: specSubset(spec, (method) => !READ_METHODS.includes(method)),
});

/** One `p(95)` threshold per operation (`op` tag) of `coverage`. */
function latencyThresholds(coverage, budgetMs) {
  const thresholds = {};
  for (const operation of coverage.operations) {
    thresholds[`http_req_duration{op:${operation.op}}`] = [`p(95)<${budgetMs}`];
  }
  return thresholds;
}

export const options = {
  scenarios: {
    reads: {
      executor: 'ramping-vus',
      exec: 'readScenario',
      stages: [
        { duration: '30s', target: Math.ceil(PROFILE.readVus / 2) },
        { duration: '30s', target: PROFILE.readVus },
        { duration: '2m', target: PROFILE.readVus }, // Hold
        { duration: '20s', target: 0 },
      ],
    },
    writes: {
      executor: 'constant-vus',
      exec: 'writeScenario',
      vus: PROFILE.writeVus,
      duration: '3m20s',
    },
    formations_rush: {
      executor: 'constant-arrival-rate',
      exec: 'formationsRushScenario',
      startTime: '1m', // once the reads are at full load
      rate: FORMATIONS_RUSH_RATE,
      timeUnit: '1s',
      duration: '1m',
      preAllocatedVUs: 50,
      maxVUs: 200,
    },
  },
  thresholds: {
    ...reads.thresholds, // every operation exercised, no handler error (shared counters)
    ...latencyThresholds(reads, READ_BUDGET_MS),
    ...latencyThresholds(writes, WRITE_BUDGET_MS),
    'http_req_duration{op:formations_rush}': [`p(95)<${FORMATIONS_RUSH_BUDGET_MS}`],
    dropped_iterations: ['count==0'], // the rush kept its rate
    // Strict (MAIR-474): one wrong status or one missing seeded row fails the run.
    checks: ['rate==1'],
    http_req_failed: ['rate==0'],
  },
};

export function readScenario() {
  reads.run({ headers: AUTH });
  sleep(1);
}

export function writeScenario() {
  writes.run({ headers: AUTH });
  sleep(1);
}

export function formationsRushScenario() {
  const res = http.get(`${BASE_URL}/api/v1/formations/`, {
    headers: randomLearner().headers,
    tags: { op: 'formations_rush' },
  });
  check(res, {
    'formations rush 200': (r) => r.status === 200,
    'formations rush reads the enrolments': (r) =>
      r.status === 200 && r.json('formations').length >= ENROLMENTS_PER_LEARNER,
  });
}
