-- Volume seed of the performance stack (MAIR-474), run by the `seeder` service after
-- init-test.sql. Without it k6 reads two formations and a few dozen users, so the costs that grow
-- with the catalogue (admin lists paged with OFFSET over nested aggregates, a learner's progress
-- across their formations) are never measured.
--
-- - 300 formations (ids 5000..5299) of 10 modules each (module 10000 + 10 * f + m, `f` the
--   formation's rank, `m` 0..9) and 2 attachments per module (20000 + 2 * module rank + a);
-- - 2 000 learners (`User`, ids 200001..202000). Learner `i` (rank 0..1999) is enrolled in the 15
--   formations 5000 + (i + 20 * k) % 300, k = 0..14, and has completed the first 5 modules of each
--   (150 000 completions). load-test.js derives the same ids to read as a learner.
--
-- Fixed ids, ON CONFLICT DO NOTHING: the file is idempotent, like init-test.sql.

INSERT INTO users (id, first_name, last_name, email, password, status, is_archived)
SELECT n, 'Apprenant', 'Perf ' || n, 'perf.learner.' || n || '@mairie360.fr',
       '$argon2id$v=19$m=19456,t=2,p=1$/iKF9PbiDRDs4EKPjlIIhg$UKx9vfwwps250mEP/bYp63CXbEnQGULeUAhDq+az9Aw',
       'active', FALSE
FROM generate_series(200001, 202000) AS n
ON CONFLICT (id) DO NOTHING;

INSERT INTO user_roles (user_id, role_id)
SELECT n, r.id FROM generate_series(200001, 202000) AS n CROSS JOIN roles r WHERE r.name = 'User'
ON CONFLICT DO NOTHING;

INSERT INTO courses (id, title, description, category, level)
SELECT 5000 + f, 'Formation perf ' || f, 'Performance seed',
       (ARRAY['RGPD', 'Accueil', 'Urbanisme', 'Finances'])[1 + f % 4],
       (ARRAY['beginner', 'intermediate', 'advanced'])[1 + f % 3]
FROM generate_series(0, 299) AS f
ON CONFLICT (id) DO NOTHING;

INSERT INTO course_modules (id, course_id, title, content, sort_order)
SELECT 10000 + 10 * f + m, 5000 + f, 'Module ' || m, 'Performance seed', m + 1
FROM generate_series(0, 299) AS f CROSS JOIN generate_series(0, 9) AS m
ON CONFLICT (id) DO NOTHING;

INSERT INTO course_attachments (id, module_id, title, file_name, file_type, file_url, file_size_bytes)
SELECT 20000 + 2 * (10 * f + m) + a, 10000 + 10 * f + m, 'Support ' || a, 'support-' || a || '.pdf',
       'pdf', 'elearning/perf/' || f || '/' || m || '/support-' || a || '.pdf', 482913
FROM generate_series(0, 299) AS f CROSS JOIN generate_series(0, 9) AS m CROSS JOIN generate_series(0, 1) AS a
ON CONFLICT (id) DO NOTHING;

INSERT INTO user_courses (user_id, course_id, status, started_at)
SELECT 200001 + i, 5000 + (i + 20 * k) % 300, 'in_progress', now() - k * interval '1 day'
FROM generate_series(0, 1999) AS i CROSS JOIN generate_series(0, 14) AS k
ON CONFLICT DO NOTHING;

INSERT INTO user_modules (user_id, module_id, is_completed, completed_at)
SELECT 200001 + i, 10000 + 10 * ((i + 20 * k) % 300) + m, TRUE, now() - m * interval '1 hour'
FROM generate_series(0, 1999) AS i CROSS JOIN generate_series(0, 14) AS k
CROSS JOIN generate_series(0, 4) AS m
ON CONFLICT DO NOTHING;

SELECT setval(pg_get_serial_sequence('users', 'id'), GREATEST((SELECT MAX(id) FROM users), 1));
SELECT setval(pg_get_serial_sequence('courses', 'id'), GREATEST((SELECT MAX(id) FROM courses), 1));
SELECT setval(pg_get_serial_sequence('course_modules', 'id'), GREATEST((SELECT MAX(id) FROM course_modules), 1));
SELECT setval(pg_get_serial_sequence('course_attachments', 'id'), GREATEST((SELECT MAX(id) FROM course_attachments), 1));

ANALYZE users;
ANALYZE user_roles;
ANALYZE courses;
ANALYZE course_modules;
ANALYZE course_attachments;
ANALYZE user_courses;
ANALYZE user_modules;
