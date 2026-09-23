-- Fixtures for the newman integration tests (docker-compose-integration.yml `seeder` service).
-- The API has no route to create courses, modules or attachments, so they are seeded here with
-- fixed ids; user 2 is the plain agent the collection registers and progresses. Everything is
-- idempotent and user 2's progress is reset so the scenario replays on a persistent database.

INSERT INTO users (id, first_name, last_name, email, password, status)
VALUES (2, 'Test', 'User', 'test2@mairie360.fr', 'dummy', 'active')
ON CONFLICT (id) DO NOTHING;

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
