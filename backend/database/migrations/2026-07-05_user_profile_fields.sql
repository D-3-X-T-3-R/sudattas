-- Customer profile self-service fields: first/last name, gender, date of birth.

SET @users_first_name_col_exists := (
  SELECT COUNT(1)
  FROM information_schema.columns
  WHERE table_schema = DATABASE()
    AND table_name = 'Users'
    AND column_name = 'first_name'
);
SET @users_add_first_name_col_sql := IF(
  @users_first_name_col_exists = 0,
  'ALTER TABLE `Users` ADD COLUMN `first_name` VARCHAR(100) NULL',
  'SELECT 1'
);
PREPARE stmt FROM @users_add_first_name_col_sql;
EXECUTE stmt;
DEALLOCATE PREPARE stmt;

SET @users_last_name_col_exists := (
  SELECT COUNT(1)
  FROM information_schema.columns
  WHERE table_schema = DATABASE()
    AND table_name = 'Users'
    AND column_name = 'last_name'
);
SET @users_add_last_name_col_sql := IF(
  @users_last_name_col_exists = 0,
  'ALTER TABLE `Users` ADD COLUMN `last_name` VARCHAR(100) NULL',
  'SELECT 1'
);
PREPARE stmt FROM @users_add_last_name_col_sql;
EXECUTE stmt;
DEALLOCATE PREPARE stmt;

SET @users_gender_col_exists := (
  SELECT COUNT(1)
  FROM information_schema.columns
  WHERE table_schema = DATABASE()
    AND table_name = 'Users'
    AND column_name = 'gender'
);
SET @users_add_gender_col_sql := IF(
  @users_gender_col_exists = 0,
  'ALTER TABLE `Users` ADD COLUMN `gender` ENUM(''male'',''female'',''other'') NULL',
  'SELECT 1'
);
PREPARE stmt FROM @users_add_gender_col_sql;
EXECUTE stmt;
DEALLOCATE PREPARE stmt;

SET @users_date_of_birth_col_exists := (
  SELECT COUNT(1)
  FROM information_schema.columns
  WHERE table_schema = DATABASE()
    AND table_name = 'Users'
    AND column_name = 'date_of_birth'
);
SET @users_add_date_of_birth_col_sql := IF(
  @users_date_of_birth_col_exists = 0,
  'ALTER TABLE `Users` ADD COLUMN `date_of_birth` DATE NULL',
  'SELECT 1'
);
PREPARE stmt FROM @users_add_date_of_birth_col_sql;
EXECUTE stmt;
DEALLOCATE PREPARE stmt;
