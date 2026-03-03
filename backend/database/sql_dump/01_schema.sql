SET character_set_client = utf8mb4;

CREATE DATABASE IF NOT EXISTS `SUDATTAS`;
USE `SUDATTAS`;

-- Dropping existing tables if they exist;
-- Dropping tables with dependencies first

DROP TABLE IF EXISTS `outbox_events`;
DROP TABLE IF EXISTS `order_events`;
DROP TABLE IF EXISTS `webhook_events`;
DROP TABLE IF EXISTS `shipments`;
DROP TABLE IF EXISTS `payment_intents`;
DROP TABLE IF EXISTS `idempotency_keys`;
DROP TABLE IF EXISTS `sessions`;
DROP TABLE IF EXISTS `coupons`;
DROP TABLE IF EXISTS `OrderDetails`;
DROP TABLE IF EXISTS `Cart`;
DROP TABLE IF EXISTS `Wishlist`;
DROP TABLE IF EXISTS `ProductImages`;
DROP TABLE IF EXISTS `Reviews`;
DROP TABLE IF EXISTS `Transactions`;
DROP TABLE IF EXISTS `NewsletterSubscribers`;
DROP TABLE IF EXISTS `InventoryLog`;
DROP TABLE IF EXISTS `UserActivity`;
DROP TABLE IF EXISTS `EventLogs`;
DROP TABLE IF EXISTS `Inventory`;
DROP TABLE IF EXISTS `Orders`;
DROP TABLE IF EXISTS `ProductVariants`;
DROP TABLE IF EXISTS `Products`;
DROP TABLE IF EXISTS `ProductCategories`;
DROP TABLE IF EXISTS `ProductAttributes`;
DROP TABLE IF EXISTS `ShippingMethods`;
DROP TABLE IF EXISTS `UserRoles`;
DROP TABLE IF EXISTS `Sizes`;
DROP TABLE IF EXISTS `Colors`;
DROP TABLE IF EXISTS `ShippingAddresses`;
DROP TABLE IF EXISTS `OrderStatus`;
DROP TABLE IF EXISTS `security_audit_log`;
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
    `role_id` BIGINT DEFAULT NULL,
    `last_login_at` TIMESTAMP NULL,
    `marketing_opt_out` BOOLEAN DEFAULT FALSE COMMENT 'P2: do not send abandoned-cart / marketing emails',
    `CreateDate` timestamp NOT NULL DEFAULT CURRENT_TIMESTAMP,
    `updated_at` TIMESTAMP DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    PRIMARY KEY (`UserID`),
    INDEX `idx_email` (`Email`),
    INDEX `idx_user_status` (`user_status_id`),
    INDEX `idx_user_role` (`role_id`),
    CONSTRAINT `fk_users_user_status`
      FOREIGN KEY (`user_status_id`) REFERENCES `user_statuses`(`id`),
    CONSTRAINT `fk_users_role`
      FOREIGN KEY (`role_id`) REFERENCES `UserRoles`(`RoleID`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_general_ci;

-- Table structure for table `ProductCategories`
CREATE TABLE `ProductCategories` (
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
    `CategoryID` bigint NOT NULL,
    `attribute_ids` JSON NOT NULL COMMENT 'Array of ProductAttributes.AttributeID values, e.g. [1,2] for Katha + Block print',
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
    FOREIGN KEY (`CategoryID`) REFERENCES `ProductCategories`(`CategoryID`),
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

-- Create the ShippingAddresses table (flattened for D2C: no country/state/city lookup tables)
CREATE TABLE `ShippingAddresses` (
    `ShippingAddressID` BIGINT NOT NULL AUTO_INCREMENT,
    `Country` VARCHAR(100) NOT NULL,
    `StateRegion` VARCHAR(100) NOT NULL,
    `City` VARCHAR(100) NOT NULL,
    `PostalCode` VARCHAR(20) NOT NULL,
    `Road` VARCHAR(255),
    `ApartmentNoOrName` VARCHAR(255),
    PRIMARY KEY (`ShippingAddressID`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_general_ci;

-- Table structure for table `Orders` (Enhanced with payment tracking and order/coupon snapshot)
CREATE TABLE `Orders` (
    `OrderID` BIGINT NOT NULL AUTO_INCREMENT,
    `order_number` VARCHAR(50) UNIQUE COMMENT 'SUD-2024-00001',
    `UserID` BIGINT NOT NULL,
    `OrderDate` TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    `ShippingAddressID` BIGINT NOT NULL,
    `TotalAmount` DECIMAL(10,2) NOT NULL COMMENT 'Legacy field, use total_paise instead',
    `StatusID` BIGINT NOT NULL,
    `payment_status` ENUM('pending', 'authorized', 'captured', 'failed', 'needs_review') DEFAULT 'pending',
    `payment_method` VARCHAR(50) NULL DEFAULT NULL,
    `currency` VARCHAR(3) DEFAULT 'INR',
    `updated_at` TIMESTAMP DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    `subtotal_minor` BIGINT NULL DEFAULT NULL,
    `shipping_minor` BIGINT NULL DEFAULT 0,
    `tax_total_minor` BIGINT NULL DEFAULT 0,
    `discount_total_minor` BIGINT NULL DEFAULT 0,
    `grand_total_minor` BIGINT NULL DEFAULT NULL,
    `applied_coupon_id` BIGINT NULL DEFAULT NULL,
    `applied_coupon_code` VARCHAR(64) NULL DEFAULT NULL,
    `applied_discount_paise` INT NULL DEFAULT NULL,
    PRIMARY KEY (`OrderID`),
    FOREIGN KEY (`UserID`) REFERENCES `Users`(`UserID`),
    FOREIGN KEY (`ShippingAddressID`) REFERENCES `ShippingAddresses`(`ShippingAddressID`),
    FOREIGN KEY (`StatusID`) REFERENCES `OrderStatus`(`StatusID`),
    INDEX `idx_order_number` (`order_number`),
    INDEX `idx_payment_status` (`payment_status`),
    INDEX `idx_orders_date_status_user` (`OrderDate`, `StatusID`, `UserID`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_general_ci;

-- Table structure for table `OrderDetails` (with line-level snapshot)
CREATE TABLE `OrderDetails` (
    `OrderDetailID` bigint NOT NULL AUTO_INCREMENT,
    `OrderID` bigint NOT NULL,
    `VariantID` bigint NOT NULL,
    `Quantity` bigint NOT NULL,
    `Price` decimal(10,2) NOT NULL,
    `unit_price_minor` INT NULL DEFAULT NULL,
    `discount_minor` INT NULL DEFAULT NULL,
    `tax_minor` INT NULL DEFAULT NULL,
    `sku` VARCHAR(255) NULL DEFAULT NULL,
    `title` VARCHAR(512) NULL DEFAULT NULL,
    `line_attrs` JSON NULL DEFAULT NULL,
    PRIMARY KEY (`OrderDetailID`),
    FOREIGN KEY (`OrderID`) REFERENCES `Orders`(`OrderID`),
    FOREIGN KEY (`VariantID`) REFERENCES `ProductVariants`(`VariantID`),
    INDEX `idx_order_details_order_id` (`OrderID`)
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
    `VariantID` bigint NOT NULL,
    `Quantity` bigint NOT NULL,
    `created_at` TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    `updated_at` TIMESTAMP DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    `abandoned_email_sent_at` TIMESTAMP NULL COMMENT 'P2: when set, do not send another abandoned-cart email',
    PRIMARY KEY (`CartID`),
    FOREIGN KEY (`UserID`) REFERENCES `Users`(`UserID`),
    FOREIGN KEY (`VariantID`) REFERENCES `ProductVariants`(`VariantID`),
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
    `urls` JSON NOT NULL COMMENT 'JSON array of CDN image URLs for this product, ordered [1,2,3,...]',
    `created_at` TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (`ImageID`),
    FOREIGN KEY (`ProductID`) REFERENCES `Products`(`ProductID`),
    INDEX `idx_product_order` (`ProductID`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_general_ci;

-- Table structure for table `Inventory` (Enhanced with reserved stock)
CREATE TABLE `Inventory` (
    `InventoryID` bigint NOT NULL AUTO_INCREMENT,
    `VariantID` bigint DEFAULT NULL,
    `QuantityAvailable` bigint DEFAULT NULL,
    `quantity_reserved` INT DEFAULT 0 COMMENT 'Reserved for pending orders',
    `ReorderLevel` bigint DEFAULT NULL,
    `updated_at` TIMESTAMP DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    PRIMARY KEY (`InventoryID`),
    FOREIGN KEY (`VariantID`) REFERENCES `ProductVariants`(`VariantID`),
    INDEX `idx_inventory_variant_id` (`VariantID`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_general_ci;

-- Table structure for table `ProductAttributes`
CREATE TABLE `ProductAttributes` (
    `AttributeID` bigint NOT NULL AUTO_INCREMENT,
    `AttributeName` varchar(255) DEFAULT NULL,
    `AttributeValue` varchar(255) DEFAULT NULL,
    PRIMARY KEY (`AttributeID`)
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
    `unsubscribed_at` TIMESTAMP NULL COMMENT 'P2: when set, do not send newsletters',
    PRIMARY KEY (`SubscriberID`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_general_ci;

-- Table structure for table `UserRoleMapping`
-- Table structure for table `Sizes`
CREATE TABLE `Sizes` (
    `SizeID` bigint NOT NULL AUTO_INCREMENT,
    `SizeName` varchar(50) NOT NULL,
    PRIMARY KEY (`SizeID`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_general_ci;

-- Table structure for table `Colors`
CREATE TABLE `Colors` (
    `ColorID` bigint NOT NULL AUTO_INCREMENT,
    `ColorName` varchar(50) NOT NULL,
    PRIMARY KEY (`ColorID`)
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

-- Table structure for table `InventoryLog` (P1: actor + before/after for admin audit)
CREATE TABLE `InventoryLog` (
    `LogID` bigint NOT NULL AUTO_INCREMENT,
    `ProductID` bigint NOT NULL,
    `ChangeQuantity` bigint NOT NULL,
    `LogTime` timestamp NOT NULL DEFAULT CURRENT_TIMESTAMP,
    `Reason` varchar(255) DEFAULT NULL,
    `actor_id` varchar(255) DEFAULT NULL COMMENT 'P1 admin audit: who changed',
    `quantity_before` bigint DEFAULT NULL COMMENT 'P1 admin audit',
    `quantity_after` bigint DEFAULT NULL COMMENT 'P1 admin audit',
    PRIMARY KEY (`LogID`),
    FOREIGN KEY (`ProductID`) REFERENCES `Products`(`ProductID`)
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
    `status` ENUM('pending', 'processed', 'failed', 'needs_review', 'client_verified') NOT NULL DEFAULT 'pending',
    `razorpay_payment_id` VARCHAR(100) NULL,
    `metadata` JSON,
    `created_at` TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    `expires_at` TIMESTAMP NOT NULL,
    `gateway_fee_paise` INT NULL COMMENT 'P1 Settlement: fee from gateway if provided',
    `gateway_tax_paise` INT NULL COMMENT 'P1 Settlement: tax from gateway if provided',
    FOREIGN KEY (`order_id`) REFERENCES `Orders`(`OrderID`),
    FOREIGN KEY (`user_id`) REFERENCES `Users`(`UserID`),
    UNIQUE KEY `uq_razorpay_payment_id` (`razorpay_payment_id`),
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
    `max_uses_per_customer` INT NULL COMMENT 'If set, each user may use this coupon at most this many times',
    `coupon_status` ENUM('active', 'inactive') DEFAULT 'active',
    `starts_at` TIMESTAMP NOT NULL,
    `ends_at` TIMESTAMP NULL,
    `created_at` TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    INDEX `idx_code` (`code`, `coupon_status`),
    INDEX `idx_coupons_code_ends_at` (`code`, `ends_at`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_general_ci;

-- P1 Coupons & promotions: per-customer redemption tracking (recorded on verified payment).
CREATE TABLE `coupon_redemptions` (
    `redemption_id` BIGINT PRIMARY KEY AUTO_INCREMENT,
    `coupon_id` BIGINT NOT NULL,
    `user_id` BIGINT NOT NULL,
    `order_id` BIGINT NOT NULL,
    `redeemed_at` TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (`coupon_id`) REFERENCES `coupons`(`coupon_id`),
    FOREIGN KEY (`user_id`) REFERENCES `Users`(`UserID`),
    FOREIGN KEY (`order_id`) REFERENCES `Orders`(`OrderID`),
    INDEX `idx_coupon_user` (`coupon_id`, `user_id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_general_ci;

-- P1 Coupons & promotions: allowlist/denylist product or category applicability.
CREATE TABLE `coupon_scope` (
    `id` BIGINT PRIMARY KEY AUTO_INCREMENT,
    `coupon_id` BIGINT NOT NULL,
    `scope_type` ENUM('product', 'category') NOT NULL,
    `scope_id` BIGINT NOT NULL,
    `is_allowlist` TINYINT(1) NOT NULL COMMENT '1=allow (cart must match at least one), 0=deny (cart must not match any)',
    UNIQUE KEY `uq_coupon_scope` (`coupon_id`, `scope_type`, `scope_id`),
    FOREIGN KEY (`coupon_id`) REFERENCES `coupons`(`coupon_id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_general_ci;

-- P1 Payments & refunds: refund records (gateway_refund_id unique for idempotency).
CREATE TABLE `refunds` (
    `refund_id` BIGINT PRIMARY KEY AUTO_INCREMENT,
    `order_id` BIGINT NOT NULL,
    `gateway_refund_id` VARCHAR(100) UNIQUE NOT NULL COMMENT 'Idempotency: same id returns same refund',
    `amount_paise` INT NOT NULL,
    `currency` VARCHAR(3) DEFAULT 'INR',
    `status` ENUM('pending', 'processed', 'failed', 'needs_review', 'client_verified') DEFAULT 'pending',
    `line_items_refunded` JSON NULL COMMENT 'Optional: [{order_detail_id, quantity_refunded, amount_paise}]',
    `created_at` TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (`order_id`) REFERENCES `Orders`(`OrderID`),
    INDEX `idx_refunds_order` (`order_id`),
    INDEX `idx_refunds_gateway` (`gateway_refund_id`)
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

-- Webhook events for idempotent webhook processing (webhook_id unique per provider)
CREATE TABLE `webhook_events` (
    `event_id` BIGINT PRIMARY KEY AUTO_INCREMENT,
    `provider` VARCHAR(50) NOT NULL,
    `event_type` VARCHAR(100) NOT NULL,
    `webhook_id` VARCHAR(255) NOT NULL,
    `provider_event_id` VARCHAR(255) NULL UNIQUE COMMENT 'e.g. x-razorpay-event-id for replay protection',
    `payload` JSON NOT NULL,
    `status` ENUM('pending', 'processed', 'failed', 'needs_review', 'client_verified') DEFAULT 'pending',
    `received_at` TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    UNIQUE KEY `uq_webhook_provider_id` (`provider`, `webhook_id`),
    INDEX `idx_webhook_id` (`webhook_id`),
    INDEX `idx_status` (`status`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_general_ci;

-- Idempotency keys for durable place_order / capture_payment
CREATE TABLE `idempotency_keys` (
    `id` BIGINT NOT NULL AUTO_INCREMENT,
    `scope` VARCHAR(255) NOT NULL,
    `key` VARCHAR(255) NOT NULL,
    `request_hash` VARCHAR(255) NOT NULL,
    `response_ref` VARCHAR(255) NULL DEFAULT NULL,
    `status` ENUM('pending', 'processed', 'failed', 'needs_review', 'client_verified') NOT NULL,
    `created_at` TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    `expires_at` TIMESTAMP NOT NULL,
    PRIMARY KEY (`id`),
    UNIQUE INDEX `idx_idempotency_scope_key` (`scope`, `key`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_general_ci;

-- P2 Security: audit log for secrets rotation and other security events
CREATE TABLE `security_audit_log` (
    `id` BIGINT NOT NULL AUTO_INCREMENT,
    `event_type` VARCHAR(100) NOT NULL COMMENT 'e.g. secrets_rotation, config_reload',
    `details` TEXT NULL,
    `created_at` TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (`id`),
    INDEX `idx_security_audit_created` (`created_at`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_general_ci;

-- P1 Outbox for transactional notifications (emails/SMS); worker publishes pending idempotently
CREATE TABLE `outbox_events` (
    `event_id` BIGINT NOT NULL AUTO_INCREMENT,
    `event_type` VARCHAR(50) NOT NULL COMMENT 'OrderPlaced, PaymentCaptured, Shipped, Delivered, Refunded',
    `aggregate_type` VARCHAR(50) NOT NULL DEFAULT 'order',
    `aggregate_id` VARCHAR(255) NOT NULL,
    `payload` JSON NOT NULL,
    `status` ENUM('pending', 'processed', 'failed', 'needs_review', 'client_verified') NOT NULL DEFAULT 'pending' COMMENT 'processed = published for delivery',
    `created_at` TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    `published_at` TIMESTAMP NULL,
    PRIMARY KEY (`event_id`),
    INDEX `idx_outbox_status` (`status`),
    INDEX `idx_outbox_created` (`created_at`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_general_ci;

-- ============================================================================
-- DEFAULT DATA
-- ============================================================================

-- P1 Data model: FK from Orders to coupons (coupons table created after Orders)
ALTER TABLE `Orders`
  ADD CONSTRAINT `fk_orders_applied_coupon`
  FOREIGN KEY (`applied_coupon_id`) REFERENCES `coupons`(`coupon_id`);

-- Insert default order statuses
INSERT INTO `OrderStatus` (`StatusName`) VALUES
('pending'),
('confirmed'),
('processing'),
('shipped'),
('delivered'),
('cancelled'),
('refunded'),
('needs_review')
ON DUPLICATE KEY UPDATE StatusName = VALUES(StatusName);
