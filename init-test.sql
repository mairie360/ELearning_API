-- Fixtures run by the `seeder` service of docker-compose-integration.yml (newman),
-- docker-compose-security.yml (ZAP) and docker-compose-performance.yml (k6). The API creates
-- courses with links, quizzes and other contents since MAIR-817, but not stored files (no upload):
-- the courses the journeys read are seeded here with fixed ids. User 1 (Admin) is created by the
-- `create_admin` changeset of the liquibase-migrations image and is the `sub` of the JWT ZAP
-- injects; user 2 is the plain agent the collection registers and progresses. Everything is
-- idempotent and user 2's progress is reset so the scenario replays on a persistent database.
--
-- The password is the public argon2id hash of the template admin account: `users.password` only
-- accepts argon2id PHC strings since database 1.3.0 (chk_users_password_hashed).

INSERT INTO users (id, first_name, last_name, email, password, status, is_archived)
VALUES
    (2, 'Test', 'User', 'test2@mairie360.fr',
     '$argon2id$v=19$m=19456,t=2,p=1$/iKF9PbiDRDs4EKPjlIIhg$UKx9vfwwps250mEP/bYp63CXbEnQGULeUAhDq+az9Aw',
     'active', FALSE),
    (42, 'Jean', 'Dupont', 'jean.dupont@mairie360.fr',
     '$argon2id$v=19$m=19456,t=2,p=1$/iKF9PbiDRDs4EKPjlIIhg$UKx9vfwwps250mEP/bYp63CXbEnQGULeUAhDq+az9Aw',
     'active', FALSE)
ON CONFLICT (id) DO NOTHING;

-- Users 3001-3064 are the agents of the k6 write VUs (load-test.js, `AGENT_BASE_ID + VU id`): each
-- write VU enrols and unenrols its own agent, since unenrolling an agent twice answers 404.
INSERT INTO users (id, first_name, last_name, email, password, status, is_archived)
SELECT n, 'Agent', 'K6 ' || n, 'k6-agent-' || n || '@mairie360.fr',
       '$argon2id$v=19$m=19456,t=2,p=1$/iKF9PbiDRDs4EKPjlIIhg$UKx9vfwwps250mEP/bYp63CXbEnQGULeUAhDq+az9Aw',
       'active', FALSE
FROM generate_series(3001, 3064) AS n
ON CONFLICT (id) DO NOTHING;

INSERT INTO user_roles (user_id, role_id)
SELECT u.id, r.id FROM roles r
CROSS JOIN (SELECT 2 UNION ALL SELECT 42 UNION ALL SELECT generate_series(3001, 3064)) AS u(id)
WHERE r.name = 'User'
ON CONFLICT DO NOTHING;

-- Explicit ids do not advance the SERIAL sequence: move it past the fixtures so users created
-- later do not collide with them.
SELECT setval(pg_get_serial_sequence('users', 'id'), GREATEST((SELECT MAX(id) FROM users), 1));

INSERT INTO courses (id, title, description)
VALUES (1000, 'Formation RGPD Collectivités (newman)', 'Comprendre les enjeux du RGPD en mairie.')
ON CONFLICT (id) DO NOTHING;

-- The catalogue exposes the deadline of a formation (MAIR-1110): the newman journey checks it.
UPDATE courses SET deadline = DATE '2026-12-15' WHERE id = 1000;

INSERT INTO course_modules (id, course_id, title, content, sort_order) VALUES
    (1001, 1000, 'Introduction et définitions', 'Licéité, minimisation, durée de conservation', 1),
    (1002, 1000, 'Les obligations de la collectivité', 'Registre des traitements et DPO', 2)
ON CONFLICT (id) DO NOTHING;

INSERT INTO course_attachments (id, module_id, title, file_name, file_type, file_url, file_size_bytes)
VALUES (1003, 1001, 'Guide RGPD', 'guide_rgpd_mairie.pdf', 'pdf', 'elearning/guides/guide_rgpd_mairie.pdf', 482913)
ON CONFLICT (id) DO NOTHING;

-- MAIR-1150: metadata of the V2 catalogue, and durations for the training time and attestations.
UPDATE courses SET category = 'Protection des données', level = 'beginner' WHERE id IN (4, 1000);
UPDATE course_attachments SET duration_seconds = 600 WHERE id = 1003;

DELETE FROM user_modules WHERE (user_id = 2 OR user_id BETWEEN 3001 AND 3064)
    AND module_id IN (1001, 1002);
DELETE FROM course_ratings WHERE user_id = 2 AND course_id = 1000;
DELETE FROM user_courses WHERE (user_id = 2 OR user_id BETWEEN 3001 AND 3064)
    AND course_id = 1000;

-- Formation 4, module 11, attachment 27 and user 42 (enrolled, first module done) are the ids of
-- the path parameter examples of the spec: ZAP builds its requests from these examples, so
-- seeding them makes it scan the handlers on real rows instead of stopping at a 404.
INSERT INTO courses (id, title, description)
VALUES (4, 'RGPD pour les agents territoriaux', 'Obligations et bonnes pratiques')
ON CONFLICT (id) DO NOTHING;

INSERT INTO course_modules (id, course_id, title, content, sort_order)
VALUES (11, 4, 'Les principes du RGPD', 'Licéité, minimisation, durée de conservation', 1)
ON CONFLICT (id) DO NOTHING;

INSERT INTO course_attachments (id, module_id, title, file_name, file_type, file_url, file_size_bytes)
VALUES (27, 11, 'Principes du RGPD', 'rgpd-principes.pdf', 'pdf', 'elearning/rgpd-principes.pdf', 482913)
ON CONFLICT (id) DO NOTHING;

INSERT INTO user_courses (user_id, course_id) VALUES (42, 4)
ON CONFLICT DO NOTHING;

INSERT INTO user_modules (user_id, module_id, is_completed, completed_at)
VALUES (42, 11, TRUE, '2026-09-03 14:25:00')
ON CONFLICT DO NOTHING;

-- The Admin (user 1) is the scanning user of ZAP and k6. Since MAIR-223 the routes under
-- /api/v1/formations/{formation_id}/ answer 403 to a caller who is not enrolled in the
-- formation, admins included. Enrol the Admin in every formation the stacks hit (4 from the
-- spec examples, 1000 from load-test.js) so those routes return 2xx and the coverage gate does
-- not see only 403 responses. k6's teardown() unenrols the Admin from 1000; the next seeding
-- enrols it again.
INSERT INTO user_courses (user_id, course_id) VALUES (1, 4), (1, 1000)
ON CONFLICT DO NOTHING;

-- MAIR-1149: the Admin completed formation 4 (its only module), so the rating, the attestation and
-- the attestations of the year answer 2xx to the scanning user of ZAP and k6. The progress trigger
-- marks the enrolment completed. A replay dates the completion again, so the attestations of the
-- current year (no `year`, load-test.js) still find it after a new year.
INSERT INTO user_modules (user_id, module_id, is_completed, completed_at)
VALUES (1, 11, TRUE, CURRENT_TIMESTAMP)
ON CONFLICT (user_id, module_id) DO UPDATE SET is_completed = TRUE, completed_at = EXCLUDED.completed_at;
UPDATE course_attachments SET duration_seconds = 900 WHERE id = 27;

-- MAIR-1150: formation 1010 is mandatory and its deadline is past; user 42 is enrolled and has
-- not completed it, so the reminders route has a late agent to remind (newman, k6) and the
-- indicators list one. Nothing resets it: a reminder is only one more row.
INSERT INTO courses (id, title, description, category, is_mandatory, deadline)
VALUES (1010, 'Sécurité incendie (obligatoire)', 'Consignes et évacuation', 'Sécurité', TRUE, '2020-01-31')
ON CONFLICT (id) DO NOTHING;
INSERT INTO course_modules (id, course_id, title, content, sort_order)
VALUES (1011, 1010, 'Les consignes', 'Alerte et évacuation', 1)
ON CONFLICT (id) DO NOTHING;
INSERT INTO user_courses (user_id, course_id) VALUES (42, 1010)
ON CONFLICT DO NOTHING;

-- Formation 7 is the path example of `PATCH` / `DELETE /api/v1/admin/formations/{formation_id}/`:
-- ZAP replaces and deletes it, so it is a formation nothing else reads.
INSERT INTO courses (id, title, description)
VALUES (7, 'Formation jetable (scan)', 'Remplacée puis supprimée par le scan ZAP')
ON CONFLICT (id) DO NOTHING;

-- Explicit ids do not advance the SERIAL sequences: move them past the fixtures so the courses,
-- modules and contents created by the API (MAIR-817) do not collide with them.
SELECT setval(pg_get_serial_sequence('courses', 'id'), GREATEST((SELECT MAX(id) FROM courses), 1));
SELECT setval(pg_get_serial_sequence('course_modules', 'id'), GREATEST((SELECT MAX(id) FROM course_modules), 1));
SELECT setval(pg_get_serial_sequence('course_attachments', 'id'), GREATEST((SELECT MAX(id) FROM course_attachments), 1));
