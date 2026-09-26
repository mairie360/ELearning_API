// k6 load test, run by ./performance_test.sh (docker-compose-performance.yml).
//
// Built on the shared OpenAPI coverage module (mairie360/CICD `tests/k6/coverage.js`, MAIR-194):
// every operation of the spec served by the API needs exactly one handler ("METHOD /path", path
// as in openapi.json), k6 aborts at init otherwise. When you add an endpoint, add its handler to
// `readHandlers` (GET) or `writeHandlers` (any other method) and send its request through
// `request()` (raw `http.*` calls are not counted).
//
// Two scenarios share the spec, split by HTTP method:
// - `reads`: the GET operations under the historical profile (ramp up to 20 VUs);
// - `writes`: every other operation with 2 VUs, each handler undoing what it did when the API
//   allows it.
//
// The API cannot create formations, modules or attachments: they come from init-test.sql
// (formation 1000, modules 1001-1002, attachment 1003 of module 1001). setup() enrolls the Admin
// in formation 1000 so the "my formations" routes have something to return; teardown()
// unenrolls it. Completing a module has no inverse route: the Admin's completion of module 1002
// stays (the write is an idempotent upsert, so the table does not grow).
import http from 'k6/http';
import { check, fail, sleep } from 'k6';
import { createCoverage, loadSpec } from '/coverage.js';

const BASE_URL = (__ENV.BASE_URL || 'http://localhost:3006').replace(/\/+$/, '');

// Static HS256 JWT (sub=1, the Admin seeded by liquibase, role=admin, exp=2100, signed with the
// stack's JWT_SECRET=b"secret"), the same one ZAP injects.
const TOKEN =
  __ENV.JWT ||
  'eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxIiwicm9sZSI6ImFkbWluIiwiZXhwIjo0MTAyNDQ0ODAwfQ.xCeBe_2QxRlXW8WXr3t6F69wbEHA93HbP_7l4OTJwjA';
const AUTH = { Authorization: `Bearer ${TOKEN}` };

const ADMIN_ID = 1;
// Plain `User` account seeded by init-test.sql, enrolled and unenrolled by the writes.
const AGENT_ID = 2;
// Fixtures of init-test.sql.
const FORMATION_ID = 1000;
const MODULE_ID = 1001;
const ATTACHMENT_ID = 1003;
const MODULE_TO_COMPLETE_ID = 1002;

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

  // Management view.
  'GET /api/v1/admin/formations/': ({ request }) =>
    check(request({ query: { details: true } }), { 'catalog 200': (r) => r.status === 200 }),
  'GET /api/v1/admin/formations/{formation_id}/': ({ request }) =>
    check(request({ path: { formation_id: FORMATION_ID }, query: { details: true } }), {
      'catalog formation 200': (r) => r.status === 200,
    }),
  'GET /api/v1/admin/users/': ({ request }) =>
    check(request(), { 'learners 200': (r) => r.status === 200 }),
  'GET /api/v1/admin/users/{user_id}/': ({ request }) =>
    check(request({ path: { user_id: ADMIN_ID }, query: { details: true } }), {
      'learner 200': (r) => r.status === 200,
    }),
  'GET /api/v1/admin/users/{user_id}/{formation_id}/': ({ request }) =>
    check(request({ path: { user_id: ADMIN_ID, formation_id: FORMATION_ID }, query: { details: true } }), {
      'learner formation 200': (r) => r.status === 200,
    }),

  // View of the caller (the Admin, enrolled by setup()).
  'GET /api/v1/formations/': ({ request }) =>
    check(request(), { 'my formations 200': (r) => r.status === 200 }),
  'GET /api/v1/formations/{formation_id}/': ({ request }) =>
    check(request({ path: { formation_id: FORMATION_ID } }), {
      'my formation 200': (r) => r.status === 200,
    }),
  'GET /api/v1/formations/{formation_id}/{module_id}/': ({ request }) =>
    check(request({ path: { formation_id: FORMATION_ID, module_id: MODULE_ID } }), {
      'my module 200': (r) => r.status === 200,
    }),
  'GET /api/v1/formations/{formation_id}/{module_id}/{attachment_id}/': ({ request }) =>
    check(
      request({ path: { formation_id: FORMATION_ID, module_id: MODULE_ID, attachment_id: ATTACHMENT_ID } }),
      { 'attachment url 200': (r) => r.status === 200 },
    ),
};

const writeHandlers = {
  'POST /': ({ request }) => check(request(), { 'hello 200': (r) => r.status === 200 }),

  // Enrollment: enroll → unenroll (both idempotent, so the two VUs never conflict).
  'POST /api/v1/admin/formations/{formation_id}/': ({ request }) => {
    check(request({ path: { formation_id: FORMATION_ID }, body: { user_id: AGENT_ID } }), {
      'enroll 200': (r) => r.status === 200,
    });
    unenroll(AGENT_ID, FORMATION_ID);
  },
  'DELETE /api/v1/admin/users/{user_id}/{formation_id}/': ({ request }) => {
    enroll(AGENT_ID, FORMATION_ID);
    check(request({ path: { user_id: AGENT_ID, formation_id: FORMATION_ID } }), {
      'unenroll 204': (r) => r.status === 204,
    });
  },

  // Progress of the caller (no inverse route, see the header).
  'PATCH /api/v1/formations/{formation_id}/{module_id}/': ({ request }) =>
    check(request({ path: { formation_id: FORMATION_ID, module_id: MODULE_TO_COMPLETE_ID } }), {
      'complete module 200': (r) => r.status === 200,
    }),
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
        { duration: '30s', target: 20 }, // Ramp up to 20 virtual users
        { duration: '1m', target: 20 }, // Hold
        { duration: '10s', target: 0 }, // Ramp down
      ],
    },
    writes: {
      executor: 'constant-vus',
      exec: 'writeScenario',
      vus: 2,
      duration: '1m40s',
    },
  },
  thresholds: {
    ...reads.thresholds, // every operation exercised, no handler error (shared counters)
    ...latencyThresholds(reads, READ_BUDGET_MS),
    ...latencyThresholds(writes, WRITE_BUDGET_MS),
    http_req_failed: ['rate<0.01'], // Less than 1% errors
  },
};

export function setup() {
  enroll(ADMIN_ID, FORMATION_ID);
}

export function teardown() {
  unenroll(ADMIN_ID, FORMATION_ID);
}

export function readScenario() {
  reads.run({ headers: AUTH });
  sleep(1);
}

export function writeScenario() {
  writes.run({ headers: AUTH });
  sleep(1);
}
