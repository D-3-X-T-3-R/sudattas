-- One-time bootstrap: map a Google user (by sub claim) to admin role.
-- Usage:
--   1) Replace the placeholder Google sub in the WHERE clause.
--   2) Execute once against your target DB.
--   3) Optionally remove ADMIN_ALLOWED_USER_IDS fallback after validation.

INSERT IGNORE INTO `UserRoles` (`RoleName`) VALUES ('admin');

UPDATE `Users` u
JOIN `UserRoles` r ON r.`RoleName` = 'admin'
SET u.`role_id` = r.`RoleID`
WHERE u.`google_sub` = 'REPLACE_WITH_GOOGLE_SUB';

SELECT u.`UserID`, u.`Email`, u.`google_sub`, r.`RoleName`
FROM `Users` u
LEFT JOIN `UserRoles` r ON u.`role_id` = r.`RoleID`
WHERE u.`google_sub` = 'REPLACE_WITH_GOOGLE_SUB';

