-- Ensure canonical admin roles exist.
INSERT IGNORE INTO `UserRoles` (`RoleName`) VALUES
  ('admin'),
  ('super_admin'),
  ('customer');

