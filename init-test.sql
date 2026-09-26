-- Fixtures run by the `seeder` service of docker-compose-integration.yml (newman) and
-- docker-compose-security.yml (ZAP). The API has no route to create courses, modules or
-- attachments, so they are seeded here with fixed ids. User 1 (Admin) is created by the
-- `create_admin` changeset of the liquibase-migrations image and is the `sub` of the JWT ZAP
-- injects; user 2 is the plain agent the collection registers and progresses. Everything is
-- idempotent and user 2's progress is reset so the scenario replays on a persistent database.
--
-- The password is the public argon2id hash of the template admin account: `users.password` only
-- accepts argon2id PHC strings since database 1.3.0 (chk_users_password_hashed).

INSERT INTO users (id, first_name, last_name, email, password, status, is_archived)
VALUES (
    2, 'Test', 'User', 'test2@mairie360.fr',
    '$argon2id$v=19$m=19456,t=2,p=1$/iKF9PbiDRDs4EKPjlIIhg$UKx9vfwwps250mEP/bYp63CXbEnQGULeUAhDq+az9Aw',
    'active', FALSE
)
ON CONFLICT (id) DO NOTHING;

INSERT INTO user_roles (user_id, role_id)
SELECT 2, id FROM roles WHERE name = 'User'
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
