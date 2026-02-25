SET character_set_client = utf8mb4;

CREATE DATABASE IF NOT EXISTS `SUDATTAS`;
USE `SUDATTAS`;

-- Dropping existing tables if they exist;
-- Dropping tables with dependencies first

DROP TABLE IF EXISTS `order_events`;
DROP TABLE IF EXISTS `webhook_events`;
DROP TABLE IF EXISTS `shipments`;
DROP TABLE IF EXISTS `payment_intents`;
DROP TABLE IF EXISTS `sessions`;
DROP TABLE IF EXISTS `coupons`;
DROP TABLE IF EXISTS `OrderDetails`;
DROP TABLE IF EXISTS `Cart`;
DROP TABLE IF EXISTS `Wishlist`;
DROP TABLE IF EXISTS `ProductImages`;
DROP TABLE IF EXISTS `Reviews`;
DROP TABLE IF EXISTS `UserRolesMapping`;
DROP TABLE IF EXISTS `Transactions`;
DROP TABLE IF EXISTS `NewsletterSubscribers`;
DROP TABLE IF EXISTS `ProductRatings`;
DROP TABLE IF EXISTS `ProductCategoryMapping`;
DROP TABLE IF EXISTS `ProductAttributeMapping`;
DROP TABLE IF EXISTS `UserRoleMapping`;
DROP TABLE IF EXISTS `ProductSizeMapping`;
DROP TABLE IF EXISTS `ProductColorMapping`;
DROP TABLE IF EXISTS `InventoryLog`;
DROP TABLE IF EXISTS `UserActivity`;
DROP TABLE IF EXISTS `EventLogs`;
DROP TABLE IF EXISTS `Inventory`;
DROP TABLE IF EXISTS `Orders`;
DROP TABLE IF EXISTS `ProductVariants`;
DROP TABLE IF EXISTS `Products`;
DROP TABLE IF EXISTS `Categories`;
DROP TABLE IF EXISTS `Suppliers`;
DROP TABLE IF EXISTS `ProductAttributes`;
DROP TABLE IF EXISTS `Discounts`;
DROP TABLE IF EXISTS `ShippingMethods`;
DROP TABLE IF EXISTS `UserRoles`;
DROP TABLE IF EXISTS `Sizes`;
DROP TABLE IF EXISTS `Colors`;
DROP TABLE IF EXISTS `ShippingZones`;
DROP TABLE IF EXISTS `Promotions`;
DROP TABLE IF EXISTS `PaymentMethods`;
DROP TABLE IF EXISTS `ShippingAddresses`;
DROP TABLE IF EXISTS `StateCityMapping`;
DROP TABLE IF EXISTS `CountryStateMapping`;
DROP TABLE IF EXISTS `Cities`;
DROP TABLE IF EXISTS `States`;
DROP TABLE IF EXISTS `Countries`;
DROP TABLE IF EXISTS `OrderStatus`;
DROP TABLE IF EXISTS `Users`;

/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!40101 SET character_set_client = utf8 */;

-- ============================================================================
-- CORE TABLES (Production-Ready)
-- ============================================================================

-- Table structure for table `Users` (Enhanced with auth & security)
-- Lookup table for user statuses
CREATE TABLE `user_statuses` (
    `id` BIGINT NOT NULL AUTO_INCREMENT,
    `code` VARCHAR(50) NOT NULL UNIQUE,
    PRIMARY KEY (`id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_general_ci;

INSERT INTO `user_statuses` (`code`) VALUES
  ('active'),
  ('inactive'),
  ('suspended');

CREATE TABLE `Users` (
    `UserID` bigint NOT NULL AUTO_INCREMENT,
    `Username` varchar(255) NOT NULL,
    `Password` varchar(255) NOT NULL COMMENT 'Legacy field, use password_hash instead',
    `password_hash` varchar(255) DEFAULT NULL COMMENT 'Argon2id hash',
    `Email` varchar(255) NOT NULL UNIQUE,
    `email_verified` BOOLEAN DEFAULT FALSE,
    `email_verified_at` TIMESTAMP NULL,
    `FullName` varchar(255) DEFAULT NULL,
    `Address` text,
    `Phone` varchar(20) DEFAULT NULL,
    `user_status_id` BIGINT DEFAULT NULL,
    `last_login_at` TIMESTAMP NULL,
    `CreateDate` timestamp NOT NULL DEFAULT CURRENT_TIMESTAMP,
    `updated_at` TIMESTAMP DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    PRIMARY KEY (`UserID`),
    INDEX `idx_email` (`Email`),
    INDEX `idx_user_status` (`user_status_id`),
    CONSTRAINT `fk_users_user_status`
      FOREIGN KEY (`user_status_id`) REFERENCES `user_statuses`(`id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_general_ci;

-- Table structure for table `Categories`
CREATE TABLE `Categories` (
    `CategoryID` bigint NOT NULL AUTO_INCREMENT,
    `Name` varchar(255) NOT NULL,
    PRIMARY KEY (`CategoryID`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_general_ci;

-- Table structure for table `Products` (Enhanced for saree e-commerce)
-- Lookup table for product statuses
CREATE TABLE `product_statuses` (
    `id` BIGINT NOT NULL AUTO_INCREMENT,
    `code` VARCHAR(50) NOT NULL UNIQUE,
    PRIMARY KEY (`id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_general_ci;

INSERT INTO `product_statuses` (`code`) VALUES
  ('draft'),
  ('active'),
  ('archived');

CREATE TABLE `Products` (
    `ProductID` bigint NOT NULL AUTO_INCREMENT,
    `sku` VARCHAR(100) UNIQUE,
    `Name` varchar(255) NOT NULL,
    `slug` VARCHAR(255) UNIQUE,
    `Description` text,
    `Price` decimal(10,2) NOT NULL COMMENT 'Legacy field, use price_paise instead',
    `price_paise` INT DEFAULT NULL COMMENT 'Price in paise (₹499.00 = 49900)',
    `StockQuantity` bigint DEFAULT NULL,
    `CategoryID` bigint DEFAULT NULL,
    -- Saree-specific fields
    `fabric` VARCHAR(100),
    `weave` VARCHAR(100),
    `occasion` VARCHAR(100),
    `length_meters` DECIMAL(3,1) DEFAULT 5.5,
    `has_blouse_piece` BOOLEAN DEFAULT TRUE,
    `care_instructions` TEXT,
    -- Product management
    `product_status_id` BIGINT DEFAULT NULL,
    `created_at` TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    `updated_at` TIMESTAMP DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    PRIMARY KEY (`ProductID`),
    FOREIGN KEY (`CategoryID`) REFERENCES `Categories`(`CategoryID`),
    INDEX `idx_sku` (`sku`),
    INDEX `idx_slug` (`slug`),
    INDEX `idx_product_status` (`product_status_id`),
    INDEX `idx_fabric` (`fabric`),
    CONSTRAINT `fk_products_product_status`
      FOREIGN KEY (`product_status_id`) REFERENCES `product_statuses`(`id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_general_ci;

-- Table structure for table `OrderStatus`
CREATE TABLE `OrderStatus` (
    `StatusID` bigint NOT NULL AUTO_INCREMENT,
    `StatusName` varchar(50) NOT NULL,
    PRIMARY KEY (`StatusID`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_general_ci;

-- Create the Countries table
CREATE TABLE `Countries` (
    `CountryID` BIGINT NOT NULL AUTO_INCREMENT,
    `CountryName` VARCHAR(100),
    PRIMARY KEY (`CountryID`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_general_ci;

-- Create the States table
CREATE TABLE `States` (
    `StateID` BIGINT NOT NULL AUTO_INCREMENT,
    `StateName` VARCHAR(100),
    PRIMARY KEY (`StateID`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_general_ci;

-- Create the Cities table
CREATE TABLE `Cities` (
    `CityID` BIGINT NOT NULL AUTO_INCREMENT,
    `CityName` VARCHAR(100),
    PRIMARY KEY (`CityID`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_general_ci;

-- Create the CountryStateMapping table
CREATE TABLE `CountryStateMapping` (
    `ID` BIGINT NOT NULL AUTO_INCREMENT,
    `CountryID` BIGINT NOT NULL,
    `StateID` BIGINT NOT NULL,
    PRIMARY KEY (`ID`),
    FOREIGN KEY (`CountryID`) REFERENCES `Countries`(`CountryID`),
    FOREIGN KEY (`StateID`) REFERENCES `States`(`StateID`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_general_ci;

-- Create the StateCityMapping table
CREATE TABLE `StateCityMapping` (
    `ID` BIGINT NOT NULL AUTO_INCREMENT,
    `StateID` BIGINT NOT NULL,
    `CityID` BIGINT NOT NULL,
    PRIMARY KEY (`ID`),
    FOREIGN KEY (`StateID`) REFERENCES `States`(`StateID`),
    FOREIGN KEY (`CityID`) REFERENCES `Cities`(`CityID`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_general_ci;

-- Create the ShippingAddresses table
CREATE TABLE `ShippingAddresses` (
    `ShippingAddressID` BIGINT NOT NULL AUTO_INCREMENT,
    `CountryID` BIGINT NOT NULL,
    `StateID` BIGINT NOT NULL,
    `CityID` BIGINT NOT NULL,
    `Road` VARCHAR(255),
    `ApartmentNoOrName` VARCHAR(255),
    PRIMARY KEY (`ShippingAddressID`),
    FOREIGN KEY (`CountryID`) REFERENCES `Countries`(`CountryID`),
    FOREIGN KEY (`StateID`) REFERENCES `States`(`StateID`),
    FOREIGN KEY (`CityID`) REFERENCES `Cities`(`CityID`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_general_ci;

-- Table structure for table `Orders` (Enhanced with payment tracking)
CREATE TABLE `Orders` (
    `OrderID` BIGINT NOT NULL AUTO_INCREMENT,
    `order_number` VARCHAR(50) UNIQUE COMMENT 'SUD-2024-00001',
    `UserID` BIGINT NOT NULL,
    `OrderDate` TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    `ShippingAddressID` BIGINT NOT NULL,
    `TotalAmount` DECIMAL(10,2) NOT NULL COMMENT 'Legacy field, use total_paise instead',
    `StatusID` BIGINT NOT NULL,
    `payment_status` ENUM('pending', 'authorized', 'captured', 'failed') DEFAULT 'pending',
    `currency` VARCHAR(3) DEFAULT 'INR',
    `updated_at` TIMESTAMP DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    PRIMARY KEY (`OrderID`),
    FOREIGN KEY (`UserID`) REFERENCES `Users`(`UserID`),
    FOREIGN KEY (`ShippingAddressID`) REFERENCES `ShippingAddresses`(`ShippingAddressID`),
    FOREIGN KEY (`StatusID`) REFERENCES `OrderStatus`(`StatusID`),
    INDEX `idx_order_number` (`order_number`),
    INDEX `idx_payment_status` (`payment_status`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_general_ci;

-- Table structure for table `OrderDetails`
CREATE TABLE `OrderDetails` (
    `OrderDetailID` bigint NOT NULL AUTO_INCREMENT,
    `OrderID` bigint NOT NULL,
    `ProductID` bigint NOT NULL,
    `Quantity` bigint NOT NULL,
    `Price` decimal(10,2) NOT NULL,
    PRIMARY KEY (`OrderDetailID`),
    FOREIGN KEY (`OrderID`) REFERENCES `Orders`(`OrderID`),
    FOREIGN KEY (`ProductID`) REFERENCES `Products`(`ProductID`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_general_ci;

-- Table structure for table `Reviews` (Enhanced with moderation)
CREATE TABLE `Reviews` (
    `ReviewID` bigint NOT NULL AUTO_INCREMENT,
    `ProductID` bigint DEFAULT NULL,
    `UserID` bigint DEFAULT NULL,
    `Rating` bigint DEFAULT NULL,
    `Comment` text,
    `status` ENUM('pending', 'approved', 'rejected') DEFAULT 'pending',
    `is_verified_purchase` BOOLEAN DEFAULT FALSE,
    `created_at` TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (`ReviewID`),
    FOREIGN KEY (`ProductID`) REFERENCES `Products`(`ProductID`),
    FOREIGN KEY (`UserID`) REFERENCES `Users`(`UserID`),
    INDEX `idx_product_status` (`ProductID`, `status`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_general_ci;

-- Table structure for table `Cart` (Enhanced with session support)
CREATE TABLE `Cart` (
    `CartID` bigint NOT NULL AUTO_INCREMENT,
    `UserID` bigint NULL COMMENT 'NULL for guest carts',
    `session_id` VARCHAR(255) NULL COMMENT 'For guest checkout',
    `ProductID` bigint NOT NULL,
    `Quantity` bigint NOT NULL,
    `created_at` TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    `updated_at` TIMESTAMP DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    PRIMARY KEY (`CartID`),
    FOREIGN KEY (`UserID`) REFERENCES `Users`(`UserID`),
    FOREIGN KEY (`ProductID`) REFERENCES `Products`(`ProductID`),
    INDEX `idx_session` (`session_id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_general_ci;

-- Table structure for table `Wishlist`
CREATE TABLE `Wishlist` (
    `WishlistID` bigint NOT NULL AUTO_INCREMENT,
    `UserID` bigint DEFAULT NULL,
    `ProductID` bigint DEFAULT NULL,
    `DateAdded` timestamp NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (`WishlistID`),
    FOREIGN KEY (`UserID`) REFERENCES `Users`(`UserID`),
    FOREIGN KEY (`ProductID`) REFERENCES `Products`(`ProductID`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_general_ci;

-- Table structure for table `ProductImages` (Enhanced for CDN)
CREATE TABLE `ProductImages` (
    `ImageID` bigint NOT NULL AUTO_INCREMENT,
    `ProductID` bigint NOT NULL,
    `ImageBase64` MEDIUMTEXT NULL COMMENT 'Legacy field, use url instead',
    `url` VARCHAR(500) COMMENT 'CDN URL: https://cdn.sudattas.com/products/...',
    `cdn_path` VARCHAR(500) COMMENT 'Path in R2: products/saree-123/hero.webp',
    `thumbnail_url` VARCHAR(500),
    `file_size_bytes` INT,
    `AltText` varchar(255) DEFAULT NULL,
    `display_order` INT DEFAULT 0,
    `created_at` TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (`ImageID`),
    FOREIGN KEY (`ProductID`) REFERENCES `Products`(`ProductID`),
    INDEX `idx_product_order` (`ProductID`, `display_order`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_general_ci;

-- Table structure for table `Suppliers`
CREATE TABLE `Suppliers` (
    `SupplierID` bigint NOT NULL AUTO_INCREMENT,
    `Name` varchar(255) DEFAULT NULL,
    `ContactInfo` varchar(255) DEFAULT NULL,
    `Address` text,
    PRIMARY KEY (`SupplierID`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_general_ci;

-- Table structure for table `Inventory` (Enhanced with reserved stock)
CREATE TABLE `Inventory` (
    `InventoryID` bigint NOT NULL AUTO_INCREMENT,
    `ProductID` bigint DEFAULT NULL,
    `QuantityAvailable` bigint DEFAULT NULL,
    `quantity_reserved` INT DEFAULT 0 COMMENT 'Reserved for pending orders',
    `ReorderLevel` bigint DEFAULT NULL,
    `SupplierID` bigint DEFAULT NULL,
    `updated_at` TIMESTAMP DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    PRIMARY KEY (`InventoryID`),
    FOREIGN KEY (`ProductID`) REFERENCES `Products`(`ProductID`),
    FOREIGN KEY (`SupplierID`) REFERENCES `Suppliers`(`SupplierID`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_general_ci;

-- Table structure for table `ProductAttributes`
CREATE TABLE `ProductAttributes` (
    `AttributeID` bigint NOT NULL AUTO_INCREMENT,
    `ProductID` bigint DEFAULT NULL,
    `AttributeName` varchar(255) DEFAULT NULL,
    `AttributeValue` varchar(255) DEFAULT NULL,
    PRIMARY KEY (`AttributeID`),
    FOREIGN KEY (`ProductID`) REFERENCES `Products`(`ProductID`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_general_ci;

-- Table structure for table `Discounts`
CREATE TABLE `Discounts` (
    `DiscountID` bigint NOT NULL AUTO_INCREMENT,
    `ProductID` bigint DEFAULT NULL,
    `DiscountPercentage` decimal(5,2) DEFAULT NULL,
    `StartDate` date DEFAULT NULL,
    `EndDate` date DEFAULT NULL,
    PRIMARY KEY (`DiscountID`),
    FOREIGN KEY (`ProductID`) REFERENCES `Products`(`ProductID`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_general_ci;

-- Table structure for table `ShippingMethods`
CREATE TABLE `ShippingMethods` (
    `MethodID` bigint NOT NULL AUTO_INCREMENT,
    `MethodName` varchar(255) DEFAULT NULL,
    `Cost` decimal(10,2) DEFAULT NULL,
    `EstimatedDeliveryTime` varchar(255) DEFAULT NULL,
    PRIMARY KEY (`MethodID`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_general_ci;

-- Table structure for table `UserRoles`
CREATE TABLE `UserRoles` (
    `RoleID` bigint NOT NULL AUTO_INCREMENT,
    `RoleName` varchar(255) NOT NULL,
    PRIMARY KEY (`RoleID`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_general_ci;

-- Table structure for table `UserRolesMapping`
CREATE TABLE `UserRolesMapping` (
    `ID` bigint NOT NULL AUTO_INCREMENT,
    `UserID` bigint NOT NULL,
    `RoleID` bigint DEFAULT NULL,
    PRIMARY KEY (`ID`),
    FOREIGN KEY (`UserID`) REFERENCES `Users`(`UserID`),
    FOREIGN KEY (`RoleID`) REFERENCES `UserRoles`(`RoleID`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_general_ci;

-- Table structure for table `Transactions`
CREATE TABLE `Transactions` (
    `TransactionID` bigint NOT NULL AUTO_INCREMENT,
    `UserID` bigint NOT NULL,
    `Amount` decimal(10,2) NOT NULL,
    `TransactionDate` timestamp NOT NULL DEFAULT CURRENT_TIMESTAMP,
    `Type` varchar(50) NOT NULL,
    PRIMARY KEY (`TransactionID`),
    FOREIGN KEY (`UserID`) REFERENCES `Users`(`UserID`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_general_ci;

-- Table structure for table `NewsletterSubscribers`
CREATE TABLE `NewsletterSubscribers` (
    `SubscriberID` bigint NOT NULL AUTO_INCREMENT,
    `Email` varchar(255) NOT NULL UNIQUE,
    `SubscriptionDate` timestamp NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (`SubscriberID`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_general_ci;

-- Table structure for table `ProductRatings`
CREATE TABLE `ProductRatings` (
    `RatingID` bigint NOT NULL AUTO_INCREMENT,
    `ProductID` bigint DEFAULT NULL,
    `UserID` bigint DEFAULT NULL,
    `Rating` bigint DEFAULT NULL,
    PRIMARY KEY (`RatingID`),
    FOREIGN KEY (`ProductID`) REFERENCES `Products`(`ProductID`),
    FOREIGN KEY (`UserID`) REFERENCES `Users`(`UserID`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_general_ci;

-- Table structure for table `ProductCategoryMapping`
CREATE TABLE `ProductCategoryMapping` (
    `ProductID` bigint NOT NULL,
    `CategoryID` bigint NOT NULL,
    PRIMARY KEY (`ProductID`, `CategoryID`),
    FOREIGN KEY (`ProductID`) REFERENCES `Products`(`ProductID`),
    FOREIGN KEY (`CategoryID`) REFERENCES `Categories`(`CategoryID`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_general_ci;

-- Table structure for table `ProductAttributeMapping`
CREATE TABLE `ProductAttributeMapping` (
    `ProductID` bigint NOT NULL,
    `AttributeID` bigint NOT NULL,
    PRIMARY KEY (`ProductID`, `AttributeID`),
    FOREIGN KEY (`ProductID`) REFERENCES `Products`(`ProductID`),
    FOREIGN KEY (`AttributeID`) REFERENCES `ProductAttributes`(`AttributeID`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_general_ci;

-- Table structure for table `UserRoleMapping`
CREATE TABLE `UserRoleMapping` (
    `UserID` bigint NOT NULL,
    `RoleID` bigint NOT NULL,
    PRIMARY KEY (`UserID`, `RoleID`),
    FOREIGN KEY (`UserID`) REFERENCES `Users`(`UserID`),
    FOREIGN KEY (`RoleID`) REFERENCES `UserRoles`(`RoleID`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_general_ci;

-- Table structure for table `Sizes`
CREATE TABLE `Sizes` (
    `SizeID` bigint NOT NULL AUTO_INCREMENT,
    `SizeName` varchar(50) NOT NULL,
    PRIMARY KEY (`SizeID`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_general_ci;

-- Table structure for table `ProductSizeMapping`
CREATE TABLE `ProductSizeMapping` (
    `ProductID` bigint NOT NULL,
    `SizeID` bigint NOT NULL,
    PRIMARY KEY (`ProductID`, `SizeID`),
    FOREIGN KEY (`ProductID`) REFERENCES `Products`(`ProductID`),
    FOREIGN KEY (`SizeID`) REFERENCES `Sizes`(`SizeID`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_general_ci;

-- Table structure for table `Colors`
CREATE TABLE `Colors` (
    `ColorID` bigint NOT NULL AUTO_INCREMENT,
    `ColorName` varchar(50) NOT NULL,
    PRIMARY KEY (`ColorID`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_general_ci;

-- Table structure for table `ProductColorMapping`
CREATE TABLE `ProductColorMapping` (
    `ProductID` bigint NOT NULL,
    `ColorID` bigint NOT NULL,
    PRIMARY KEY (`ProductID`, `ColorID`),
    FOREIGN KEY (`ProductID`) REFERENCES `Products`(`ProductID`),
    FOREIGN KEY (`ColorID`) REFERENCES `Colors`(`ColorID`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_general_ci;

-- Table structure for table `ProductVariants`
CREATE TABLE `ProductVariants` (
    `VariantID` bigint NOT NULL AUTO_INCREMENT,
    `ProductID` bigint NOT NULL,
    `SizeID` bigint DEFAULT NULL,
    `ColorID` bigint DEFAULT NULL,
    `AdditionalPrice` decimal(10,2) DEFAULT NULL,
    PRIMARY KEY (`VariantID`),
    FOREIGN KEY (`ProductID`) REFERENCES `Products`(`ProductID`),
    FOREIGN KEY (`SizeID`) REFERENCES `Sizes`(`SizeID`),
    FOREIGN KEY (`ColorID`) REFERENCES `Colors`(`ColorID`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_general_ci;

-- Table structure for table `EventLogs`
CREATE TABLE `EventLogs` (
    `LogID` bigint NOT NULL AUTO_INCREMENT,
    `EventType` varchar(255) NOT NULL,
    `EventDescription` text,
    `UserID` bigint DEFAULT NULL,
    `EventTime` timestamp NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (`LogID`),
    FOREIGN KEY (`UserID`) REFERENCES `Users`(`UserID`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_general_ci;

-- Table structure for table `UserActivity`
CREATE TABLE `UserActivity` (
    `ActivityID` bigint NOT NULL AUTO_INCREMENT,
    `UserID` bigint DEFAULT NULL,
    `ActivityType` varchar(255) NOT NULL,
    `ActivityTime` timestamp NOT NULL DEFAULT CURRENT_TIMESTAMP,
    `ActivityDetails` text,
    PRIMARY KEY (`ActivityID`),
    FOREIGN KEY (`UserID`) REFERENCES `Users`(`UserID`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_general_ci;

-- Table structure for table `InventoryLog`
CREATE TABLE `InventoryLog` (
    `LogID` bigint NOT NULL AUTO_INCREMENT,
    `ProductID` bigint NOT NULL,
    `ChangeQuantity` bigint NOT NULL,
    `LogTime` timestamp NOT NULL DEFAULT CURRENT_TIMESTAMP,
    `Reason` varchar(255) DEFAULT NULL,
    PRIMARY KEY (`LogID`),
    FOREIGN KEY (`ProductID`) REFERENCES `Products`(`ProductID`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_general_ci;

-- Table structure for table `Promotions`
CREATE TABLE `Promotions` (
    `PromotionID` bigint NOT NULL AUTO_INCREMENT,
    `PromotionName` varchar(255) NOT NULL,
    `StartDate` datetime NOT NULL,
    `EndDate` datetime NOT NULL,
    `Details` text,
    PRIMARY KEY (`PromotionID`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_general_ci;

-- Table structure for table `ShippingZones`
CREATE TABLE `ShippingZones` (
    `ZoneID` bigint NOT NULL AUTO_INCREMENT,
    `ZoneName` varchar(255) NOT NULL,
    `Description` text,
    PRIMARY KEY (`ZoneID`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_general_ci;

-- Table structure for table `PaymentMethods`
CREATE TABLE `PaymentMethods` (
    `MethodID` bigint NOT NULL AUTO_INCREMENT,
    `MethodName` varchar(255) NOT NULL,
    `Details` text,
    PRIMARY KEY (`MethodID`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_general_ci;

-- ============================================================================
-- PHASE 1: PRODUCTION TABLES (Sessions, Payments, Shipping)
-- ============================================================================

-- Sessions table for Redis-backed session management
CREATE TABLE `sessions` (
    `session_id` VARCHAR(128) PRIMARY KEY,
    `user_id` BIGINT NULL,
    `data` JSON NOT NULL,
    `ip_address` VARCHAR(45),
    `last_activity` TIMESTAMP DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    `expires_at` TIMESTAMP NOT NULL,
    `created_at` TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (`user_id`) REFERENCES `Users`(`UserID`) ON DELETE CASCADE,
    INDEX `idx_user` (`user_id`),
    INDEX `idx_expires` (`expires_at`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_general_ci;

-- Payment intents for Razorpay order tracking
CREATE TABLE `payment_intents` (
    `intent_id` BIGINT PRIMARY KEY AUTO_INCREMENT,
    `razorpay_order_id` VARCHAR(100) UNIQUE NOT NULL,
    `order_id` BIGINT NULL,
    `user_id` BIGINT NULL,
    `amount_paise` INT NOT NULL,
    `currency` VARCHAR(3) DEFAULT 'INR',
    `status` ENUM('created', 'attempted', 'paid', 'failed') DEFAULT 'created',
    `razorpay_payment_id` VARCHAR(100),
    `metadata` JSON,
    `created_at` TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    `expires_at` TIMESTAMP NOT NULL,
    FOREIGN KEY (`order_id`) REFERENCES `Orders`(`OrderID`),
    FOREIGN KEY (`user_id`) REFERENCES `Users`(`UserID`),
    INDEX `idx_razorpay_order` (`razorpay_order_id`),
    INDEX `idx_status` (`status`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_general_ci;

-- Shipments for Shiprocket tracking
CREATE TABLE `shipments` (
    `shipment_id` BIGINT PRIMARY KEY AUTO_INCREMENT,
    `order_id` BIGINT NOT NULL,
    `shiprocket_order_id` VARCHAR(100),
    `awb_code` VARCHAR(100),
    `carrier` VARCHAR(100),
    `status` ENUM('pending', 'picked_up', 'in_transit', 'delivered', 'failed') DEFAULT 'pending',
    `tracking_events` JSON,
    `created_at` TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    `delivered_at` TIMESTAMP NULL,
    FOREIGN KEY (`order_id`) REFERENCES `Orders`(`OrderID`),
    INDEX `idx_order` (`order_id`),
    INDEX `idx_awb` (`awb_code`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_general_ci;

-- Coupons for discount codes
CREATE TABLE `coupons` (
    `coupon_id` BIGINT PRIMARY KEY AUTO_INCREMENT,
    `code` VARCHAR(50) UNIQUE NOT NULL,
    `discount_type` ENUM('percentage', 'fixed_amount') NOT NULL,
    `discount_value` INT NOT NULL,
    `min_order_value_paise` INT DEFAULT 0,
    `usage_limit` INT NULL,
    `usage_count` INT DEFAULT 0,
    `status` ENUM('active', 'inactive') DEFAULT 'active',
    `starts_at` TIMESTAMP NOT NULL,
    `ends_at` TIMESTAMP NULL,
    `created_at` TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    INDEX `idx_code` (`code`, `status`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_general_ci;

-- Order events for state machine audit trail
CREATE TABLE `order_events` (
    `event_id` BIGINT PRIMARY KEY AUTO_INCREMENT,
    `order_id` BIGINT NOT NULL,
    `event_type` VARCHAR(50) NOT NULL,
    `from_status` VARCHAR(50),
    `to_status` VARCHAR(50),
    `actor_type` ENUM('customer', 'admin', 'system') NOT NULL,
    `message` TEXT,
    `created_at` TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (`order_id`) REFERENCES `Orders`(`OrderID`),
    INDEX `idx_order` (`order_id`, `created_at`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_general_ci;

-- Webhook events for idempotent webhook processing
CREATE TABLE `webhook_events` (
    `event_id` BIGINT PRIMARY KEY AUTO_INCREMENT,
    `provider` VARCHAR(50) NOT NULL,
    `event_type` VARCHAR(100) NOT NULL,
    `webhook_id` VARCHAR(255) UNIQUE NOT NULL,
    `payload` JSON NOT NULL,
    `status` ENUM('pending', 'processed', 'failed') DEFAULT 'pending',
    `received_at` TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    INDEX `idx_webhook_id` (`webhook_id`),
    INDEX `idx_status` (`status`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_general_ci;

-- ============================================================================
-- DEFAULT DATA
-- ============================================================================

-- Insert default order statuses
INSERT INTO `OrderStatus` (`StatusName`) VALUES
('pending'),
('confirmed'),
('processing'),
('shipped'),
('delivered'),
('cancelled'),
('refunded')
ON DUPLICATE KEY UPDATE StatusName = VALUES(StatusName);
