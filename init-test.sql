-- Fixtures run by the `seeder` service of docker-compose-integration.yml (newman),
-- docker-compose-security.yml (ZAP) and docker-compose-performance.yml (k6). The API has no route to create courses, modules or
-- attachments, so they are seeded here with fixed ids. User 1 (Admin) is created by the
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

INSERT INTO user_roles (user_id, role_id)
SELECT u.id, r.id FROM roles r CROSS JOIN (VALUES (2), (42)) AS u(id) WHERE r.name = 'User'
ON CONFLICT DO NOTHING;

-- Explicit ids do not advance the SERIAL sequence: move it past the fixtures so users created
-- later do not collide with them.
SELECT setval(pg_get_serial_sequence('users', 'id'), GREATEST((SELECT MAX(id) FROM users), 1));

INSERT INTO courses (id, title, description)
VALUES (1000, 'Formation RGPD Collectivités (newman)', 'Comprendre les enjeux du RGPD en mairie.')
ON CONFLICT (id) DO NOTHING;

INSERT INTO course_modules (id, course_id, title, content, sort_order) VALUES
    (1001, 1000, 'Introduction et définitions', 'Licéité, minimisation, durée de conservation', 1),
    (1002, 1000, 'Les obligations de la collectivité', 'Registre des traitements et DPO', 2)
ON CONFLICT (id) DO NOTHING;

INSERT INTO course_attachments (id, module_id, title, file_name, file_type, file_url, file_size_bytes)
VALUES (1003, 1001, 'Guide RGPD', 'guide_rgpd_mairie.pdf', 'pdf', 'elearning/guides/guide_rgpd_mairie.pdf', 482913)
ON CONFLICT (id) DO NOTHING;

DELETE FROM user_modules WHERE user_id = 2 AND module_id IN (1001, 1002);
DELETE FROM user_courses WHERE user_id = 2 AND course_id = 1000;

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
