-- MySQL dump 10.13  Distrib 9.6.0, for Linux (x86_64)
--
-- Host: 127.0.0.1    Database: SUDATTAS
-- ------------------------------------------------------
-- Server version	9.6.0

/*!40101 SET @OLD_CHARACTER_SET_CLIENT=@@CHARACTER_SET_CLIENT */;
/*!40101 SET @OLD_CHARACTER_SET_RESULTS=@@CHARACTER_SET_RESULTS */;
/*!40101 SET @OLD_COLLATION_CONNECTION=@@COLLATION_CONNECTION */;
/*!50503 SET NAMES utf8mb4 */;
/*!40103 SET @OLD_TIME_ZONE=@@TIME_ZONE */;
/*!40103 SET TIME_ZONE='+00:00' */;
/*!40014 SET @OLD_UNIQUE_CHECKS=@@UNIQUE_CHECKS, UNIQUE_CHECKS=0 */;
/*!40014 SET @OLD_FOREIGN_KEY_CHECKS=@@FOREIGN_KEY_CHECKS, FOREIGN_KEY_CHECKS=0 */;
/*!40101 SET @OLD_SQL_MODE=@@SQL_MODE, SQL_MODE='NO_AUTO_VALUE_ON_ZERO' */;
/*!40111 SET @OLD_SQL_NOTES=@@SQL_NOTES, SQL_NOTES=0 */;

--
-- Table structure for table `Cart`
--

DROP TABLE IF EXISTS `Cart`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!50503 SET character_set_client = utf8mb4 */;
CREATE TABLE `Cart` (
  `CartID` bigint NOT NULL AUTO_INCREMENT,
  `UserID` bigint DEFAULT NULL COMMENT 'NULL for guest carts',
  `session_id` varchar(255) CHARACTER SET utf8mb4 COLLATE utf8mb4_general_ci DEFAULT NULL COMMENT 'For guest checkout',
  `VariantID` bigint NOT NULL,
  `Quantity` bigint NOT NULL,
  `created_at` timestamp NULL DEFAULT CURRENT_TIMESTAMP,
  `updated_at` timestamp NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  `abandoned_email_sent_at` timestamp NULL DEFAULT NULL COMMENT 'P2: when set, do not send another abandoned-cart email',
  PRIMARY KEY (`CartID`),
  UNIQUE KEY `uq_cart_user_variant` (`UserID`,`VariantID`),
  UNIQUE KEY `uq_cart_session_variant` (`session_id`,`VariantID`),
  KEY `VariantID` (`VariantID`),
  KEY `idx_session` (`session_id`),
  CONSTRAINT `Cart_ibfk_1` FOREIGN KEY (`UserID`) REFERENCES `Users` (`UserID`) ON DELETE CASCADE,
  CONSTRAINT `Cart_ibfk_2` FOREIGN KEY (`VariantID`) REFERENCES `ProductVariants` (`VariantID`)
) ENGINE=InnoDB AUTO_INCREMENT=109 DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_general_ci;
/*!40101 SET character_set_client = @saved_cs_client */;

--
-- Dumping data for table `Cart`
--

LOCK TABLES `Cart` WRITE;
/*!40000 ALTER TABLE `Cart` DISABLE KEYS */;
INSERT INTO `Cart` VALUES (1,7,NULL,31,1,'2026-04-02 12:08:37','2026-04-02 12:08:37','2026-04-02 12:08:37'),(2,8,NULL,32,1,'2026-04-02 12:08:37','2026-04-01 11:08:38',NULL),(36,54,NULL,67,1,'2026-04-03 21:03:35','2026-04-03 21:03:35','2026-04-03 21:03:36'),(37,55,NULL,68,1,'2026-04-03 21:03:36','2026-04-02 20:03:36',NULL),(72,NULL,'7f69e98b-c26a-4696-a444-ae95d233041f',3,2,'2026-04-04 22:26:36','2026-04-04 22:31:37',NULL),(75,NULL,'7f69e98b-c26a-4696-a444-ae95d233041f',4,1,'2026-04-04 22:32:00','2026-04-04 22:32:00',NULL),(77,NULL,'7f69e98b-c26a-4696-a444-ae95d233041f',5,1,'2026-04-04 22:33:13','2026-04-04 22:33:13',NULL),(80,106,NULL,3,6,'2026-04-04 22:43:08','2026-04-05 00:19:34',NULL);
/*!40000 ALTER TABLE `Cart` ENABLE KEYS */;
UNLOCK TABLES;

--
-- Table structure for table `Colors`
--

DROP TABLE IF EXISTS `Colors`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!50503 SET character_set_client = utf8mb4 */;
CREATE TABLE `Colors` (
  `ColorID` bigint NOT NULL AUTO_INCREMENT,
  `ColorName` varchar(50) CHARACTER SET utf8mb4 COLLATE utf8mb4_general_ci NOT NULL,
  PRIMARY KEY (`ColorID`)
) ENGINE=InnoDB AUTO_INCREMENT=22 DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_general_ci;
/*!40101 SET character_set_client = @saved_cs_client */;

--
-- Dumping data for table `Colors`
--

LOCK TABLES `Colors` WRITE;
/*!40000 ALTER TABLE `Colors` DISABLE KEYS */;
INSERT INTO `Colors` VALUES (1,'Red'),(2,'Maroon'),(3,'Pink'),(4,'Orange'),(5,'Yellow'),(6,'Green'),(7,'Olive'),(8,'Blue'),(9,'Navy'),(10,'Teal'),(11,'Purple'),(12,'Violet'),(13,'Magenta'),(14,'Brown'),(15,'Beige'),(16,'Cream'),(17,'Grey'),(18,'Black'),(19,'White'),(20,'Gold'),(21,'Silver');
/*!40000 ALTER TABLE `Colors` ENABLE KEYS */;
UNLOCK TABLES;

--
-- Table structure for table `CouponRedemptions`
--

DROP TABLE IF EXISTS `CouponRedemptions`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!50503 SET character_set_client = utf8mb4 */;
CREATE TABLE `CouponRedemptions` (
  `redemption_id` bigint NOT NULL AUTO_INCREMENT,
  `coupon_id` bigint NOT NULL,
  `user_id` bigint NOT NULL,
  `order_id` bigint NOT NULL,
  `redeemed_at` timestamp NULL DEFAULT CURRENT_TIMESTAMP,
  PRIMARY KEY (`redemption_id`),
  KEY `user_id` (`user_id`),
  KEY `order_id` (`order_id`),
  KEY `idx_coupon_user` (`coupon_id`,`user_id`),
  CONSTRAINT `CouponRedemptions_ibfk_1` FOREIGN KEY (`coupon_id`) REFERENCES `Coupons` (`coupon_id`),
  CONSTRAINT `CouponRedemptions_ibfk_2` FOREIGN KEY (`user_id`) REFERENCES `Users` (`UserID`),
  CONSTRAINT `CouponRedemptions_ibfk_3` FOREIGN KEY (`order_id`) REFERENCES `Orders` (`OrderID`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_general_ci;
/*!40101 SET character_set_client = @saved_cs_client */;

--
-- Dumping data for table `CouponRedemptions`
--

LOCK TABLES `CouponRedemptions` WRITE;
/*!40000 ALTER TABLE `CouponRedemptions` DISABLE KEYS */;
/*!40000 ALTER TABLE `CouponRedemptions` ENABLE KEYS */;
UNLOCK TABLES;

--
-- Table structure for table `CouponScope`
--

DROP TABLE IF EXISTS `CouponScope`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!50503 SET character_set_client = utf8mb4 */;
CREATE TABLE `CouponScope` (
  `id` bigint NOT NULL AUTO_INCREMENT,
  `coupon_id` bigint NOT NULL,
  `scope_type` enum('product','category') CHARACTER SET utf8mb4 COLLATE utf8mb4_general_ci NOT NULL,
  `scope_id` bigint NOT NULL,
  `is_allowlist` tinyint(1) NOT NULL COMMENT '1=allow (cart must match at least one), 0=deny (cart must not match any)',
  PRIMARY KEY (`id`),
  UNIQUE KEY `uq_coupon_scope` (`coupon_id`,`scope_type`,`scope_id`),
  CONSTRAINT `CouponScope_ibfk_1` FOREIGN KEY (`coupon_id`) REFERENCES `Coupons` (`coupon_id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_general_ci;
/*!40101 SET character_set_client = @saved_cs_client */;

--
-- Dumping data for table `CouponScope`
--

LOCK TABLES `CouponScope` WRITE;
/*!40000 ALTER TABLE `CouponScope` DISABLE KEYS */;
/*!40000 ALTER TABLE `CouponScope` ENABLE KEYS */;
UNLOCK TABLES;

--
-- Table structure for table `Coupons`
--

DROP TABLE IF EXISTS `Coupons`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!50503 SET character_set_client = utf8mb4 */;
CREATE TABLE `Coupons` (
  `coupon_id` bigint NOT NULL AUTO_INCREMENT,
  `code` varchar(50) CHARACTER SET utf8mb4 COLLATE utf8mb4_general_ci NOT NULL,
  `discount_type` enum('percentage','fixed_amount') CHARACTER SET utf8mb4 COLLATE utf8mb4_general_ci NOT NULL,
  `discount_value` int NOT NULL,
  `min_order_value_paise` int DEFAULT '0',
  `usage_limit` int DEFAULT NULL,
  `usage_count` int DEFAULT '0',
  `max_uses_per_customer` int DEFAULT NULL COMMENT 'If set, each user may use this coupon at most this many times',
  `coupon_status` enum('active','inactive') CHARACTER SET utf8mb4 COLLATE utf8mb4_general_ci DEFAULT 'active',
  `starts_at` timestamp NOT NULL,
  `ends_at` timestamp NULL DEFAULT NULL,
  `created_at` timestamp NULL DEFAULT CURRENT_TIMESTAMP,
  PRIMARY KEY (`coupon_id`),
  UNIQUE KEY `code` (`code`),
  KEY `idx_code` (`code`,`coupon_status`),
  KEY `idx_coupons_code_ends_at` (`code`,`ends_at`)
) ENGINE=InnoDB AUTO_INCREMENT=11 DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_general_ci;
/*!40101 SET character_set_client = @saved_cs_client */;

--
-- Dumping data for table `Coupons`
--

LOCK TABLES `Coupons` WRITE;
/*!40000 ALTER TABLE `Coupons` DISABLE KEYS */;
/*!40000 ALTER TABLE `Coupons` ENABLE KEYS */;
UNLOCK TABLES;

--
-- Table structure for table `EventLogs`
--

DROP TABLE IF EXISTS `EventLogs`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!50503 SET character_set_client = utf8mb4 */;
CREATE TABLE `EventLogs` (
  `LogID` bigint NOT NULL AUTO_INCREMENT,
  `EventType` varchar(255) CHARACTER SET utf8mb4 COLLATE utf8mb4_general_ci NOT NULL,
  `EventDescription` text CHARACTER SET utf8mb4 COLLATE utf8mb4_general_ci,
  `UserID` bigint DEFAULT NULL,
  `EventTime` timestamp NOT NULL DEFAULT CURRENT_TIMESTAMP,
  PRIMARY KEY (`LogID`),
  KEY `UserID` (`UserID`),
  CONSTRAINT `EventLogs_ibfk_1` FOREIGN KEY (`UserID`) REFERENCES `Users` (`UserID`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_general_ci;
/*!40101 SET character_set_client = @saved_cs_client */;

--
-- Dumping data for table `EventLogs`
--

LOCK TABLES `EventLogs` WRITE;
/*!40000 ALTER TABLE `EventLogs` DISABLE KEYS */;
/*!40000 ALTER TABLE `EventLogs` ENABLE KEYS */;
UNLOCK TABLES;

--
-- Table structure for table `Fabrics`
--

DROP TABLE IF EXISTS `Fabrics`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!50503 SET character_set_client = utf8mb4 */;
CREATE TABLE `Fabrics` (
  `FabricID` bigint NOT NULL AUTO_INCREMENT,
  `Name` varchar(100) CHARACTER SET utf8mb4 COLLATE utf8mb4_general_ci NOT NULL,
  PRIMARY KEY (`FabricID`),
  UNIQUE KEY `uq_fabric_name` (`Name`)
) ENGINE=InnoDB AUTO_INCREMENT=21 DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_general_ci;
/*!40101 SET character_set_client = @saved_cs_client */;

--
-- Dumping data for table `Fabrics`
--

LOCK TABLES `Fabrics` WRITE;
/*!40000 ALTER TABLE `Fabrics` DISABLE KEYS */;
INSERT INTO `Fabrics` VALUES (18,'Art Silk'),(17,'Banarasi Silk'),(13,'Brocade'),(4,'Chiffon'),(1,'Cotton'),(20,'Cotton Silk'),(6,'Crepe'),(5,'Georgette'),(14,'Khadi'),(3,'Linen'),(15,'Muslin'),(8,'Net'),(7,'Organza'),(19,'Polyester'),(9,'Rayon'),(11,'Satin'),(2,'Silk'),(16,'Tussar Silk'),(12,'Velvet'),(10,'Viscose');
/*!40000 ALTER TABLE `Fabrics` ENABLE KEYS */;
UNLOCK TABLES;

--
-- Table structure for table `IdempotencyKeys`
--

DROP TABLE IF EXISTS `IdempotencyKeys`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!50503 SET character_set_client = utf8mb4 */;
CREATE TABLE `IdempotencyKeys` (
  `id` bigint NOT NULL AUTO_INCREMENT,
  `scope` varchar(255) CHARACTER SET utf8mb4 COLLATE utf8mb4_general_ci NOT NULL,
  `key` varchar(255) CHARACTER SET utf8mb4 COLLATE utf8mb4_general_ci NOT NULL,
  `request_hash` varchar(255) CHARACTER SET utf8mb4 COLLATE utf8mb4_general_ci NOT NULL,
  `response_ref` varchar(255) CHARACTER SET utf8mb4 COLLATE utf8mb4_general_ci DEFAULT NULL,
  `status` enum('pending','processed','failed','needs_review','client_verified') CHARACTER SET utf8mb4 COLLATE utf8mb4_general_ci NOT NULL,
  `created_at` timestamp NOT NULL DEFAULT CURRENT_TIMESTAMP,
  `expires_at` timestamp NOT NULL,
  PRIMARY KEY (`id`),
  UNIQUE KEY `idx_idempotency_scope_key` (`scope`,`key`)
) ENGINE=InnoDB AUTO_INCREMENT=20 DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_general_ci;
/*!40101 SET character_set_client = @saved_cs_client */;

--
-- Dumping data for table `IdempotencyKeys`
--

LOCK TABLES `IdempotencyKeys` WRITE;
/*!40000 ALTER TABLE `IdempotencyKeys` DISABLE KEYS */;
INSERT INTO `IdempotencyKeys` VALUES (1,'place_order','checkout-place-5542871b-1116-449e-8cae-7ae34eefbb0a','{\"cart\":[{\"quantity\":1,\"variant_id\":3}],\"coupon_code\":null,\"shipping_address_id\":61,\"user_id\":104}','51','processed','2026-04-04 22:39:04','2026-04-05 22:39:04'),(2,'place_order','checkout-place-8abca284-3843-4fa2-8d20-de145bc4aaf3','{\"cart\":[{\"quantity\":1,\"variant_id\":11}],\"coupon_code\":null,\"shipping_address_id\":61,\"user_id\":104}','52','processed','2026-04-04 22:46:35','2026-04-05 22:46:35'),(3,'place_order','checkout-place-4fe54ad7-69f2-4273-ae12-55c81114d8eb','{\"cart\":[{\"quantity\":1,\"variant_id\":16}],\"coupon_code\":null,\"shipping_address_id\":61,\"user_id\":104}','53','processed','2026-04-04 22:49:58','2026-04-05 22:49:58'),(4,'place_order','checkout-place-7e7e0f16-4b70-4384-8ffd-7273b3ec0bc7','{\"cart\":[{\"quantity\":1,\"variant_id\":13}],\"coupon_code\":null,\"shipping_address_id\":61,\"user_id\":104}','54','processed','2026-04-04 23:05:29','2026-04-05 23:05:29'),(5,'place_order','checkout-place-d15c0670-aa00-4053-bcbd-01d0871a3f27','{\"cart\":[{\"quantity\":1,\"variant_id\":7},{\"quantity\":1,\"variant_id\":9},{\"quantity\":1,\"variant_id\":10}],\"coupon_code\":null,\"shipping_address_id\":61,\"user_id\":104}','55','processed','2026-04-04 23:10:45','2026-04-05 23:10:45'),(6,'place_order','checkout-place-73da025d-4479-437c-9fed-d24cedfd91fb','{\"cart\":[{\"quantity\":1,\"variant_id\":3}],\"coupon_code\":null,\"shipping_address_id\":61,\"user_id\":104}','56','processed','2026-04-05 21:44:45','2026-04-06 21:44:45'),(7,'place_order','checkout-place-70dff32c-5be1-4f99-8dda-d7632a82b727','{\"cart\":[{\"quantity\":1,\"variant_id\":7},{\"quantity\":1,\"variant_id\":8}],\"coupon_code\":null,\"shipping_address_id\":61,\"user_id\":104}','57','processed','2026-04-05 22:36:07','2026-04-06 22:36:07'),(8,'place_order','checkout-place-15cee39a-40fd-45e2-8f82-b886a6e12f0a','{\"cart\":[{\"quantity\":1,\"variant_id\":7},{\"quantity\":1,\"variant_id\":8}],\"coupon_code\":null,\"shipping_address_id\":61,\"user_id\":104}','58','processed','2026-04-05 22:37:23','2026-04-06 22:37:23'),(9,'place_order','checkout-place-aea693ac-4b90-419c-9fc1-dc381413df68','{\"cart\":[{\"quantity\":1,\"variant_id\":7},{\"quantity\":1,\"variant_id\":8}],\"coupon_code\":null,\"shipping_address_id\":61,\"user_id\":104}','59','processed','2026-04-05 22:38:40','2026-04-06 22:38:40'),(10,'place_order','checkout-place-126c27c8-1b93-4851-8542-6e620ae9a8d3','{\"cart\":[{\"quantity\":1,\"variant_id\":14}],\"coupon_code\":null,\"shipping_address_id\":61,\"user_id\":104}','60','processed','2026-04-05 22:39:35','2026-04-06 22:39:35'),(11,'place_order','checkout-place-cc7893f1-b809-472c-b7cd-b2c622384f47','{\"cart\":[{\"quantity\":3,\"variant_id\":7},{\"quantity\":1,\"variant_id\":11},{\"quantity\":1,\"variant_id\":13},{\"quantity\":1,\"variant_id\":14}],\"coupon_code\":null,\"shipping_address_id\":62,\"user_id\":104}','1','processed','2026-04-09 01:26:04','2026-04-10 01:26:04'),(14,'place_order','checkout-place-4b5a8092-b6e2-4b20-929a-e3f13492b847','{\"cart\":[{\"quantity\":1,\"variant_id\":3}],\"coupon_code\":null,\"shipping_address_id\":62,\"user_id\":104}','2','processed','2026-04-09 01:35:55','2026-04-10 01:35:55'),(15,'place_order','checkout-place-56e2479d-7c93-404e-be95-4a68ee9a6204','{\"cart\":[{\"quantity\":1,\"variant_id\":3}],\"coupon_code\":null,\"shipping_address_id\":62,\"user_id\":104}','3','processed','2026-04-09 01:37:00','2026-04-10 01:37:00'),(16,'place_order','checkout-place-cc5df4ee-7e57-47c2-ae1e-d85e6e561312','{\"cart\":[{\"quantity\":3,\"variant_id\":3}],\"coupon_code\":null,\"shipping_address_id\":62,\"user_id\":104}','4','processed','2026-04-09 02:07:16','2026-04-10 02:07:16'),(17,'place_order','checkout-place-8ee076f5-04af-47c5-9d12-51c76c14fb9b','{\"cart\":[{\"quantity\":1,\"variant_id\":11}],\"coupon_code\":null,\"shipping_address_id\":62,\"user_id\":104}','5','processed','2026-04-09 02:25:27','2026-04-10 02:25:27'),(18,'place_order','checkout-place-2688adb3-df6c-4c24-8a4b-9bb94bf0ab26','{\"cart\":[{\"quantity\":3,\"variant_id\":2}],\"coupon_code\":null,\"shipping_address_id\":62,\"user_id\":104}','6','processed','2026-04-09 21:37:15','2026-04-10 21:37:15'),(19,'place_order','checkout-place-17026470-c1a1-4b34-8858-99572f57cd2b','{\"cart\":[{\"quantity\":1,\"variant_id\":2}],\"coupon_code\":null,\"shipping_address_id\":62,\"user_id\":104}','7','processed','2026-04-09 22:05:10','2026-04-10 22:05:10');
/*!40000 ALTER TABLE `IdempotencyKeys` ENABLE KEYS */;
UNLOCK TABLES;

--
-- Table structure for table `Inventory`
--

DROP TABLE IF EXISTS `Inventory`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!50503 SET character_set_client = utf8mb4 */;
CREATE TABLE `Inventory` (
  `InventoryID` bigint NOT NULL AUTO_INCREMENT,
  `VariantID` bigint DEFAULT NULL,
  `QuantityAvailable` bigint DEFAULT NULL,
  `quantity_reserved` int DEFAULT '0' COMMENT 'Reserved for pending orders',
  `ReorderLevel` bigint DEFAULT NULL,
  `updated_at` timestamp NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (`InventoryID`),
  KEY `idx_inventory_variant_id` (`VariantID`),
  CONSTRAINT `Inventory_ibfk_1` FOREIGN KEY (`VariantID`) REFERENCES `ProductVariants` (`VariantID`)
) ENGINE=InnoDB AUTO_INCREMENT=89 DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_general_ci;
/*!40101 SET character_set_client = @saved_cs_client */;

--
-- Dumping data for table `Inventory`
--

LOCK TABLES `Inventory` WRITE;
/*!40000 ALTER TABLE `Inventory` DISABLE KEYS */;
INSERT INTO `Inventory` VALUES (1,1,1,0,0,'2026-03-07 11:09:06'),(2,2,1,0,0,'2026-04-09 22:05:10'),(3,3,0,0,0,'2026-04-09 02:07:16'),(4,4,5,0,0,'2026-03-07 23:16:25'),(5,5,5,0,0,'2026-03-07 23:16:25'),(6,6,5,0,0,'2026-03-07 23:16:26'),(7,7,1,0,0,'2026-04-09 01:26:04'),(8,8,5,0,0,'2026-04-08 02:17:15'),(9,9,2,0,0,'2026-04-04 23:10:45'),(10,10,1,0,0,'2026-04-04 23:10:45'),(11,11,0,0,0,'2026-04-09 02:25:27'),(12,12,1,0,0,'2026-03-07 23:19:08'),(13,13,0,0,0,'2026-04-09 01:26:04'),(14,14,9,0,0,'2026-04-09 01:26:04'),(15,15,1,0,0,'2026-03-07 23:21:42'),(16,16,2,0,0,'2026-04-08 02:17:39'),(17,17,1,0,0,'2026-03-07 23:22:36'),(18,18,0,0,0,'2026-03-19 21:05:27'),(19,19,3,0,0,'2026-03-19 21:32:19'),(20,20,5,0,0,'2026-03-21 11:03:17'),(21,21,5,0,0,'2026-03-21 11:03:17'),(22,22,3,0,0,'2026-03-21 11:03:17'),(23,23,2,0,0,'2026-03-21 11:03:17'),(24,24,2,0,0,'2026-03-21 11:03:35'),(25,25,1,0,0,'2026-03-21 11:03:36'),(26,26,1,0,0,'2026-03-21 11:03:47'),(27,27,10,0,0,'2026-03-21 11:04:01'),(28,28,1,0,0,'2026-03-21 11:04:02'),(29,29,2,0,0,'2026-03-21 11:04:16'),(30,30,1,0,0,'2026-03-21 11:04:17');
/*!40000 ALTER TABLE `Inventory` ENABLE KEYS */;
UNLOCK TABLES;

--
-- Table structure for table `InventoryLog`
--

DROP TABLE IF EXISTS `InventoryLog`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!50503 SET character_set_client = utf8mb4 */;
CREATE TABLE `InventoryLog` (
  `LogID` bigint NOT NULL AUTO_INCREMENT,
  `VariantID` bigint NOT NULL,
  `ChangeQuantity` bigint NOT NULL,
  `LogTime` timestamp NOT NULL DEFAULT CURRENT_TIMESTAMP,
  `Reason` varchar(255) CHARACTER SET utf8mb4 COLLATE utf8mb4_general_ci DEFAULT NULL,
  `actor_id` varchar(255) CHARACTER SET utf8mb4 COLLATE utf8mb4_general_ci DEFAULT NULL COMMENT 'P1 admin audit: who changed',
  `quantity_before` bigint DEFAULT NULL COMMENT 'P1 admin audit',
  `quantity_after` bigint DEFAULT NULL COMMENT 'P1 admin audit',
  PRIMARY KEY (`LogID`),
  KEY `VariantID` (`VariantID`),
  CONSTRAINT `InventoryLog_ibfk_1` FOREIGN KEY (`VariantID`) REFERENCES `ProductVariants` (`VariantID`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_general_ci;
/*!40101 SET character_set_client = @saved_cs_client */;

--
-- Dumping data for table `InventoryLog`
--

LOCK TABLES `InventoryLog` WRITE;
/*!40000 ALTER TABLE `InventoryLog` DISABLE KEYS */;
/*!40000 ALTER TABLE `InventoryLog` ENABLE KEYS */;
UNLOCK TABLES;

--
-- Table structure for table `NewsletterSubscribers`
--

DROP TABLE IF EXISTS `NewsletterSubscribers`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!50503 SET character_set_client = utf8mb4 */;
CREATE TABLE `NewsletterSubscribers` (
  `SubscriberID` bigint NOT NULL AUTO_INCREMENT,
  `Email` varchar(255) CHARACTER SET utf8mb4 COLLATE utf8mb4_general_ci NOT NULL,
  `SubscriptionDate` timestamp NOT NULL DEFAULT CURRENT_TIMESTAMP,
  `unsubscribed_at` timestamp NULL DEFAULT NULL COMMENT 'P2: when set, do not send newsletters',
  PRIMARY KEY (`SubscriberID`),
  UNIQUE KEY `Email` (`Email`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_general_ci;
/*!40101 SET character_set_client = @saved_cs_client */;

--
-- Dumping data for table `NewsletterSubscribers`
--

LOCK TABLES `NewsletterSubscribers` WRITE;
/*!40000 ALTER TABLE `NewsletterSubscribers` DISABLE KEYS */;
/*!40000 ALTER TABLE `NewsletterSubscribers` ENABLE KEYS */;
UNLOCK TABLES;

--
-- Table structure for table `Occasions`
--

DROP TABLE IF EXISTS `Occasions`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!50503 SET character_set_client = utf8mb4 */;
CREATE TABLE `Occasions` (
  `OccasionID` bigint NOT NULL AUTO_INCREMENT,
  `Name` varchar(100) CHARACTER SET utf8mb4 COLLATE utf8mb4_general_ci NOT NULL,
  PRIMARY KEY (`OccasionID`),
  UNIQUE KEY `uq_occasion_name` (`Name`)
) ENGINE=InnoDB AUTO_INCREMENT=11 DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_general_ci;
/*!40101 SET character_set_client = @saved_cs_client */;

--
-- Dumping data for table `Occasions`
--

LOCK TABLES `Occasions` WRITE;
/*!40000 ALTER TABLE `Occasions` DISABLE KEYS */;
INSERT INTO `Occasions` VALUES (7,'Bridal'),(1,'Casual'),(8,'Daily Wear'),(9,'Evening'),(3,'Festive'),(10,'Formal'),(5,'Party'),(6,'Puja'),(4,'Wedding'),(2,'Work');
/*!40000 ALTER TABLE `Occasions` ENABLE KEYS */;
UNLOCK TABLES;

--
-- Table structure for table `OrderDetails`
--

DROP TABLE IF EXISTS `OrderDetails`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!50503 SET character_set_client = utf8mb4 */;
CREATE TABLE `OrderDetails` (
  `OrderDetailID` bigint NOT NULL AUTO_INCREMENT,
  `OrderID` bigint NOT NULL,
  `VariantID` bigint NOT NULL,
  `Quantity` bigint NOT NULL,
  `Price` decimal(10,2) DEFAULT NULL COMMENT 'Legacy field, do not use; use unit_price_minor instead',
  `unit_price_minor` int NOT NULL,
  `discount_minor` int DEFAULT NULL,
  `tax_minor` int DEFAULT NULL,
  `sku` varchar(255) CHARACTER SET utf8mb4 COLLATE utf8mb4_general_ci DEFAULT NULL,
  `title` varchar(512) CHARACTER SET utf8mb4 COLLATE utf8mb4_general_ci DEFAULT NULL,
  `line_attrs` json DEFAULT NULL,
  PRIMARY KEY (`OrderDetailID`),
  KEY `idx_order_details_order_id` (`OrderID`),
  KEY `idx_order_details_variant` (`VariantID`),
  CONSTRAINT `OrderDetails_ibfk_1` FOREIGN KEY (`OrderID`) REFERENCES `Orders` (`OrderID`),
  CONSTRAINT `OrderDetails_ibfk_2` FOREIGN KEY (`VariantID`) REFERENCES `ProductVariants` (`VariantID`)
) ENGINE=InnoDB AUTO_INCREMENT=11 DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_general_ci;
/*!40101 SET character_set_client = @saved_cs_client */;

--
-- Dumping data for table `OrderDetails`
--

LOCK TABLES `OrderDetails` WRITE;
/*!40000 ALTER TABLE `OrderDetails` DISABLE KEYS */;
INSERT INTO `OrderDetails` VALUES (1,1,7,3,2399.97,79999,NULL,NULL,NULL,'kurti_2',NULL),(2,1,11,1,799.99,79999,NULL,NULL,NULL,'kurti_3',NULL),(3,1,13,1,799.99,79999,NULL,NULL,NULL,'kurti_4',NULL),(4,1,14,1,799.99,79999,NULL,NULL,NULL,'kurti_5',NULL),(5,2,3,1,799.99,79999,NULL,NULL,NULL,'kurti_1',NULL),(6,3,3,1,799.99,79999,NULL,NULL,NULL,'kurti_1',NULL),(7,4,3,3,2399.97,79999,NULL,NULL,NULL,'kurti_1',NULL),(8,5,11,1,799.99,79999,NULL,NULL,NULL,'kurti_3',NULL),(9,6,2,3,2399.97,79999,NULL,NULL,NULL,'kurti_1',NULL),(10,7,2,1,799.99,79999,NULL,NULL,NULL,'kurti_1',NULL);
/*!40000 ALTER TABLE `OrderDetails` ENABLE KEYS */;
UNLOCK TABLES;

--
-- Table structure for table `OrderEvents`
--

DROP TABLE IF EXISTS `OrderEvents`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!50503 SET character_set_client = utf8mb4 */;
CREATE TABLE `OrderEvents` (
  `event_id` bigint NOT NULL AUTO_INCREMENT,
  `order_id` bigint NOT NULL,
  `event_type` varchar(50) CHARACTER SET utf8mb4 COLLATE utf8mb4_general_ci NOT NULL,
  `from_status` varchar(50) CHARACTER SET utf8mb4 COLLATE utf8mb4_general_ci DEFAULT NULL,
  `to_status` varchar(50) CHARACTER SET utf8mb4 COLLATE utf8mb4_general_ci DEFAULT NULL,
  `actor_type` enum('customer','admin','system') CHARACTER SET utf8mb4 COLLATE utf8mb4_general_ci NOT NULL,
  `message` text CHARACTER SET utf8mb4 COLLATE utf8mb4_general_ci,
  `created_at` timestamp NULL DEFAULT CURRENT_TIMESTAMP,
  PRIMARY KEY (`event_id`),
  KEY `idx_order` (`order_id`,`created_at`),
  CONSTRAINT `OrderEvents_ibfk_1` FOREIGN KEY (`order_id`) REFERENCES `Orders` (`OrderID`)
) ENGINE=InnoDB AUTO_INCREMENT=19 DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_general_ci;
/*!40101 SET character_set_client = @saved_cs_client */;

--
-- Dumping data for table `OrderEvents`
--

LOCK TABLES `OrderEvents` WRITE;
/*!40000 ALTER TABLE `OrderEvents` DISABLE KEYS */;
INSERT INTO `OrderEvents` VALUES (1,1,'order_placed',NULL,'processing','customer','Order 1 placed successfully','2026-04-09 01:26:04'),(2,2,'order_placed',NULL,'processing','customer','Order 2 placed successfully (courier quote: Delhivery Surface / Γò¼├┤Γö£ΓûôΓö¼Γò¥╬ô├╢┬ú╬ô├╢├▒Γò¼├┤Γö£ΓòóΓö¼├║Γò¼├┤Γö£┬½Γö£├½╬ô├▓┬╝Γö£Γöñ╬ô├╢┬ú╬ô├╗├┤╬ô├╢┬ú╬ô├▓├ª129.00)','2026-04-09 01:35:56'),(3,3,'order_placed',NULL,'processing','customer','Order 3 placed successfully (courier quote: Delhivery Surface / Γò¼├┤Γö£ΓûôΓö¼Γò¥╬ô├╢┬ú╬ô├╢├▒Γò¼├┤Γö£ΓòóΓö¼├║Γò¼├┤Γö£┬½Γö£├½╬ô├▓┬╝Γö£Γöñ╬ô├╢┬ú╬ô├╗├┤╬ô├╢┬ú╬ô├▓├ª129.00)','2026-04-09 01:37:00'),(4,4,'order_placed',NULL,'processing','customer','Order 4 placed successfully (courier quote: Shadowfax Surface 2Kg / ╬ô├▓┬╝Γö£Γöñ╬ô├╢┬ú╬ô├«├ëΓò¼├┤Γö£ΓûôΓö£Γòæ166.00)','2026-04-09 02:07:17'),(5,4,'payment_client_verified','pending','confirmed','customer','Payment verified after Razorpay checkout','2026-04-09 02:08:18'),(6,1,'status_changed','pending','confirmed','system','Order 1 status changed to 2','2026-04-09 02:11:16'),(7,1,'status_changed','confirmed','processing','system','Order 1 status changed to 3','2026-04-09 02:11:26'),(8,1,'status_changed','processing','shipped','system','Order 1 status changed to 4','2026-04-09 02:11:29'),(9,4,'status_changed','confirmed','processing','system','Order 4 status changed to 3','2026-04-09 02:12:05'),(10,4,'status_changed','processing','shipped','system','Order 4 status changed to 4','2026-04-09 02:12:09'),(11,5,'order_placed',NULL,'processing','customer','Order 5 placed successfully (courier quote: Delhivery Surface / ╬ô├▓┬╝Γö£Γöñ╬ô├╢┬ú╬ô├«├ëΓò¼├┤Γö£ΓûôΓö£Γòæ129.00)','2026-04-09 02:25:27'),(12,5,'payment_client_verified','pending','confirmed','customer','Payment verified after Razorpay checkout','2026-04-09 02:26:00'),(13,5,'status_changed','confirmed','processing','system','Order 5 status changed to 3','2026-04-09 02:26:35'),(14,6,'order_placed',NULL,'processing','customer','Order 6 placed successfully (courier quote: Shadowfax Surface 2Kg / Γò¼├┤Γö£ΓîÉ╬ô├▓├║166.00)','2026-04-09 21:37:15'),(15,6,'payment_client_verified','pending','confirmed','customer','Payment verified after Razorpay checkout','2026-04-09 21:38:07'),(16,7,'order_placed',NULL,'processing','customer','Order 7 placed successfully (courier quote: Amazon Shipping Surface 1kg / ╬ô├⌐Γòú105.00)','2026-04-09 22:05:11'),(17,7,'payment_client_verified','pending','confirmed','customer','Payment verified after Razorpay checkout','2026-04-09 22:05:40'),(18,7,'status_changed','confirmed','processing','system','Order 7 status changed to 3','2026-04-09 23:02:26');
/*!40000 ALTER TABLE `OrderEvents` ENABLE KEYS */;
UNLOCK TABLES;

--
-- Table structure for table `OrderStatus`
--

DROP TABLE IF EXISTS `OrderStatus`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!50503 SET character_set_client = utf8mb4 */;
CREATE TABLE `OrderStatus` (
  `StatusID` bigint NOT NULL AUTO_INCREMENT,
  `StatusName` varchar(50) CHARACTER SET utf8mb4 COLLATE utf8mb4_general_ci NOT NULL,
  PRIMARY KEY (`StatusID`),
  UNIQUE KEY `uq_order_status_name` (`StatusName`)
) ENGINE=InnoDB AUTO_INCREMENT=9 DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_general_ci;
/*!40101 SET character_set_client = @saved_cs_client */;

--
-- Dumping data for table `OrderStatus`
--

LOCK TABLES `OrderStatus` WRITE;
/*!40000 ALTER TABLE `OrderStatus` DISABLE KEYS */;
INSERT INTO `OrderStatus` VALUES (6,'cancelled'),(2,'confirmed'),(5,'delivered'),(8,'needs_review'),(1,'pending'),(3,'processing'),(7,'refunded'),(4,'shipped');
/*!40000 ALTER TABLE `OrderStatus` ENABLE KEYS */;
UNLOCK TABLES;

--
-- Table structure for table `Orders`
--

DROP TABLE IF EXISTS `Orders`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!50503 SET character_set_client = utf8mb4 */;
CREATE TABLE `Orders` (
  `OrderID` bigint NOT NULL AUTO_INCREMENT,
  `order_number` varchar(50) CHARACTER SET utf8mb4 COLLATE utf8mb4_general_ci DEFAULT NULL COMMENT 'SUD-2024-00001',
  `UserID` bigint NOT NULL,
  `OrderDate` timestamp NOT NULL DEFAULT CURRENT_TIMESTAMP,
  `ShippingAddressID` bigint NOT NULL,
  `TotalAmount` decimal(10,2) DEFAULT NULL COMMENT 'Legacy field, do not use; use *_minor columns instead',
  `StatusID` bigint NOT NULL,
  `payment_status` enum('pending','authorized','captured','failed','needs_review') CHARACTER SET utf8mb4 COLLATE utf8mb4_general_ci DEFAULT 'pending',
  `payment_method` varchar(50) CHARACTER SET utf8mb4 COLLATE utf8mb4_general_ci DEFAULT NULL,
  `currency` varchar(3) CHARACTER SET utf8mb4 COLLATE utf8mb4_general_ci DEFAULT 'INR',
  `updated_at` timestamp NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  `subtotal_minor` bigint NOT NULL,
  `shipping_minor` bigint DEFAULT '0',
  `tax_total_minor` bigint DEFAULT '0',
  `discount_total_minor` bigint DEFAULT '0',
  `grand_total_minor` bigint NOT NULL,
  `applied_coupon_id` bigint DEFAULT NULL,
  `applied_coupon_code` varchar(64) CHARACTER SET utf8mb4 COLLATE utf8mb4_general_ci DEFAULT NULL,
  `applied_discount_paise` int DEFAULT NULL,
  PRIMARY KEY (`OrderID`),
  UNIQUE KEY `order_number` (`order_number`),
  KEY `UserID` (`UserID`),
  KEY `ShippingAddressID` (`ShippingAddressID`),
  KEY `StatusID` (`StatusID`),
  KEY `idx_order_number` (`order_number`),
  KEY `idx_payment_status` (`payment_status`),
  KEY `idx_orders_date_status_user` (`OrderDate`,`StatusID`,`UserID`),
  KEY `fk_orders_applied_coupon` (`applied_coupon_id`),
  CONSTRAINT `fk_orders_applied_coupon` FOREIGN KEY (`applied_coupon_id`) REFERENCES `Coupons` (`coupon_id`),
  CONSTRAINT `Orders_ibfk_1` FOREIGN KEY (`UserID`) REFERENCES `Users` (`UserID`),
  CONSTRAINT `Orders_ibfk_2` FOREIGN KEY (`ShippingAddressID`) REFERENCES `ShippingAddresses` (`ShippingAddressID`),
  CONSTRAINT `Orders_ibfk_3` FOREIGN KEY (`StatusID`) REFERENCES `OrderStatus` (`StatusID`)
) ENGINE=InnoDB AUTO_INCREMENT=8 DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_general_ci;
/*!40101 SET character_set_client = @saved_cs_client */;

--
-- Dumping data for table `Orders`
--

LOCK TABLES `Orders` WRITE;
/*!40000 ALTER TABLE `Orders` DISABLE KEYS */;
INSERT INTO `Orders` VALUES (1,NULL,104,'2026-04-09 02:11:29',62,4799.94,4,'pending',NULL,'INR','2026-04-09 02:11:29',479994,0,0,0,479994,NULL,NULL,NULL),(2,NULL,104,'2026-04-09 01:35:55',62,928.99,1,'pending',NULL,'INR','2026-04-09 01:35:55',79999,12900,0,0,92899,NULL,NULL,NULL),(3,NULL,104,'2026-04-09 01:37:00',62,928.99,1,'pending',NULL,'INR','2026-04-09 01:37:00',79999,12900,0,0,92899,NULL,NULL,NULL),(4,NULL,104,'2026-04-09 02:12:09',62,2565.97,4,'captured',NULL,'INR','2026-04-09 02:12:08',239997,16600,0,0,256597,NULL,NULL,NULL),(5,NULL,104,'2026-04-09 02:26:35',62,928.99,3,'captured',NULL,'INR','2026-04-09 02:26:34',79999,12900,0,0,92899,NULL,NULL,NULL),(6,NULL,104,'2026-04-09 21:37:15',62,2565.97,2,'captured',NULL,'INR','2026-04-09 21:38:06',239997,16600,0,0,256597,NULL,NULL,NULL),(7,NULL,104,'2026-04-09 23:02:26',62,904.99,3,'captured',NULL,'INR','2026-04-09 23:02:26',79999,10500,0,0,90499,NULL,NULL,NULL);
/*!40000 ALTER TABLE `Orders` ENABLE KEYS */;
UNLOCK TABLES;

--
-- Table structure for table `OutboxEvents`
--

DROP TABLE IF EXISTS `OutboxEvents`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!50503 SET character_set_client = utf8mb4 */;
CREATE TABLE `OutboxEvents` (
  `event_id` bigint NOT NULL AUTO_INCREMENT,
  `event_type` varchar(50) CHARACTER SET utf8mb4 COLLATE utf8mb4_general_ci NOT NULL COMMENT 'OrderPlaced, PaymentCaptured, Shipped, Delivered, Refunded',
  `aggregate_type` varchar(50) CHARACTER SET utf8mb4 COLLATE utf8mb4_general_ci NOT NULL DEFAULT 'order',
  `aggregate_id` varchar(255) CHARACTER SET utf8mb4 COLLATE utf8mb4_general_ci NOT NULL,
  `payload` json NOT NULL,
  `status` enum('pending','processed','failed','needs_review','client_verified') CHARACTER SET utf8mb4 COLLATE utf8mb4_general_ci NOT NULL DEFAULT 'pending' COMMENT 'processed = published for delivery',
  `created_at` timestamp NOT NULL DEFAULT CURRENT_TIMESTAMP,
  `published_at` timestamp NULL DEFAULT NULL,
  PRIMARY KEY (`event_id`),
  KEY `idx_outbox_status` (`status`),
  KEY `idx_outbox_created` (`created_at`)
) ENGINE=InnoDB AUTO_INCREMENT=91 DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_general_ci;
/*!40101 SET character_set_client = @saved_cs_client */;

--
-- Dumping data for table `OutboxEvents`
--

LOCK TABLES `OutboxEvents` WRITE;
/*!40000 ALTER TABLE `OutboxEvents` DISABLE KEYS */;
INSERT INTO `OutboxEvents` VALUES (1,'OrderPlaced','order','999998','{\"user_id\": 1, \"order_id\": 999998}','processed','2026-04-02 12:08:14','2026-04-02 12:08:14'),(2,'AbandonedCart','cart','7','{\"email\": \"itest_ac1+1775131716876@example.com\", \"user_id\": 7}','processed','2026-04-02 12:08:37','2026-04-03 21:07:23'),(41,'AbandonedCart','cart','54','{\"email\": \"itest_ac1+1775250215166@example.com\", \"user_id\": 54}','processed','2026-04-03 21:03:36','2026-04-03 21:07:23'),(80,'OrderPlaced','order','999998','{\"user_id\": 1, \"order_id\": 999998}','processed','2026-04-03 21:07:23','2026-04-03 21:07:23'),(81,'OrderPlaced','order','51','{\"user_id\": 104, \"order_id\": 51}','processed','2026-04-04 22:39:05','2026-04-05 11:42:18'),(82,'OrderPlaced','order','52','{\"user_id\": 104, \"order_id\": 52}','processed','2026-04-04 22:46:35','2026-04-05 11:42:18'),(83,'OrderPlaced','order','53','{\"user_id\": 104, \"order_id\": 53}','processed','2026-04-04 22:49:58','2026-04-05 11:42:18'),(84,'OrderPlaced','order','54','{\"user_id\": 104, \"order_id\": 54}','processed','2026-04-04 23:05:29','2026-04-05 11:42:18'),(85,'OrderPlaced','order','55','{\"user_id\": 104, \"order_id\": 55}','processed','2026-04-04 23:10:46','2026-04-05 11:42:18'),(86,'OrderPlaced','order','56','{\"user_id\": 104, \"order_id\": 56}','processed','2026-04-05 21:44:45','2026-04-05 21:45:00'),(87,'PaymentCaptured','order','4','{\"user_id\": 104, \"order_id\": 4}','processed','2026-04-09 02:08:18','2026-04-09 02:08:34'),(88,'PaymentCaptured','order','5','{\"user_id\": 104, \"order_id\": 5}','processed','2026-04-09 02:26:00','2026-04-09 02:26:10'),(89,'PaymentCaptured','order','6','{\"user_id\": 104, \"order_id\": 6}','processed','2026-04-09 21:38:07','2026-04-09 21:38:13'),(90,'PaymentCaptured','order','7','{\"user_id\": 104, \"order_id\": 7}','processed','2026-04-09 22:05:40','2026-04-09 22:06:10');
/*!40000 ALTER TABLE `OutboxEvents` ENABLE KEYS */;
UNLOCK TABLES;

--
-- Table structure for table `PaymentIntents`
--

DROP TABLE IF EXISTS `PaymentIntents`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!50503 SET character_set_client = utf8mb4 */;
CREATE TABLE `PaymentIntents` (
  `intent_id` bigint NOT NULL AUTO_INCREMENT,
  `razorpay_order_id` varchar(100) CHARACTER SET utf8mb4 COLLATE utf8mb4_general_ci NOT NULL,
  `order_id` bigint DEFAULT NULL,
  `user_id` bigint DEFAULT NULL,
  `amount_paise` int NOT NULL,
  `currency` varchar(3) CHARACTER SET utf8mb4 COLLATE utf8mb4_general_ci DEFAULT 'INR',
  `status` enum('pending','processed','failed','needs_review','client_verified') CHARACTER SET utf8mb4 COLLATE utf8mb4_general_ci NOT NULL DEFAULT 'pending',
  `razorpay_payment_id` varchar(100) CHARACTER SET utf8mb4 COLLATE utf8mb4_general_ci DEFAULT NULL,
  `metadata` json DEFAULT NULL,
  `created_at` timestamp NULL DEFAULT CURRENT_TIMESTAMP,
  `expires_at` timestamp NOT NULL,
  `gateway_fee_paise` int DEFAULT NULL COMMENT 'P1 Settlement: fee from gateway if provided',
  `gateway_tax_paise` int DEFAULT NULL COMMENT 'P1 Settlement: tax from gateway if provided',
  PRIMARY KEY (`intent_id`),
  UNIQUE KEY `razorpay_order_id` (`razorpay_order_id`),
  UNIQUE KEY `uq_razorpay_payment_id` (`razorpay_payment_id`),
  KEY `order_id` (`order_id`),
  KEY `user_id` (`user_id`),
  KEY `idx_razorpay_order` (`razorpay_order_id`),
  KEY `idx_status` (`status`),
  CONSTRAINT `PaymentIntents_ibfk_1` FOREIGN KEY (`order_id`) REFERENCES `Orders` (`OrderID`),
  CONSTRAINT `PaymentIntents_ibfk_2` FOREIGN KEY (`user_id`) REFERENCES `Users` (`UserID`)
) ENGINE=InnoDB AUTO_INCREMENT=15 DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_general_ci;
/*!40101 SET character_set_client = @saved_cs_client */;

--
-- Dumping data for table `PaymentIntents`
--

LOCK TABLES `PaymentIntents` WRITE;
/*!40000 ALTER TABLE `PaymentIntents` DISABLE KEYS */;
INSERT INTO `PaymentIntents` VALUES (1,'order_SbDDfXoL0Sfuro',1,104,479994,'INR','pending',NULL,NULL,'2026-04-09 01:26:04','2026-04-10 01:26:04',NULL,NULL),(2,'order_SbDDfjWwKU0PAs',1,104,479994,'INR','pending',NULL,NULL,'2026-04-09 01:26:05','2026-04-10 01:26:05',NULL,NULL),(3,'order_SbDO4kyOFjWkO3',2,104,92899,'INR','pending',NULL,NULL,'2026-04-09 01:35:56','2026-04-10 01:35:56',NULL,NULL),(4,'order_SbDO4wU1LeSWix',2,104,92899,'INR','pending',NULL,NULL,'2026-04-09 01:35:56','2026-04-10 01:35:56',NULL,NULL),(5,'order_SbDPDh0MWpvmFu',3,104,92899,'INR','pending',NULL,NULL,'2026-04-09 01:37:00','2026-04-10 01:37:00',NULL,NULL),(6,'order_SbDPDql5GBTguh',3,104,92899,'INR','pending',NULL,NULL,'2026-04-09 01:37:01','2026-04-10 01:37:01',NULL,NULL),(7,'order_SbDvCelpRErDj5',4,104,256597,'INR','pending',NULL,NULL,'2026-04-09 02:07:17','2026-04-10 02:07:17',NULL,NULL),(8,'order_SbDvCqtGbJV8YZ',4,104,256597,'INR','processed','pay_SbDvcPTrLpPdS7',NULL,'2026-04-09 02:07:17','2026-04-10 02:07:17',NULL,NULL),(9,'order_SbEEOsPMz5pEjw',5,104,92899,'INR','pending',NULL,NULL,'2026-04-09 02:25:27','2026-04-10 02:25:27',NULL,NULL),(10,'order_SbEEP3gsxofl1k',5,104,92899,'INR','processed','pay_SbEEdHTKn3IWeH',NULL,'2026-04-09 02:25:28','2026-04-10 02:25:28',NULL,NULL),(11,'order_SbXr5ajE4vxyoJ',6,104,256597,'INR','pending',NULL,NULL,'2026-04-09 21:37:15','2026-04-10 21:37:15',NULL,NULL),(12,'order_SbXr5nH222wmXV',6,104,256597,'INR','processed','pay_SbXrgPUbpm9IVs',NULL,'2026-04-09 21:37:15','2026-04-10 21:37:15',NULL,NULL),(13,'order_SbYKaRf3FQ12pg',7,104,90499,'INR','pending',NULL,NULL,'2026-04-09 22:05:11','2026-04-10 22:05:11',NULL,NULL),(14,'order_SbYKadGowRxsfA',7,104,90499,'INR','processed','pay_SbYKnrmpYGxFkE',NULL,'2026-04-09 22:05:11','2026-04-10 22:05:11',NULL,NULL);
/*!40000 ALTER TABLE `PaymentIntents` ENABLE KEYS */;
UNLOCK TABLES;

--
-- Table structure for table `ProductCategories`
--

DROP TABLE IF EXISTS `ProductCategories`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!50503 SET character_set_client = utf8mb4 */;
CREATE TABLE `ProductCategories` (
  `CategoryID` bigint NOT NULL AUTO_INCREMENT,
  `Name` varchar(255) CHARACTER SET utf8mb4 COLLATE utf8mb4_general_ci NOT NULL,
  PRIMARY KEY (`CategoryID`),
  UNIQUE KEY `uq_category_name` (`Name`)
) ENGINE=InnoDB AUTO_INCREMENT=105 DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_general_ci;
/*!40101 SET character_set_client = @saved_cs_client */;

--
-- Dumping data for table `ProductCategories`
--

LOCK TABLES `ProductCategories` WRITE;
/*!40000 ALTER TABLE `ProductCategories` DISABLE KEYS */;
INSERT INTO `ProductCategories` VALUES (13,'Accessories'),(11,'Bags & Clutches'),(5,'Blouses'),(7,'Casual Wear'),(3,'Dresses'),(6,'Ethnic Sets'),(12,'Footwear'),(8,'Formal Wear'),(15,'Gifting'),(14,'Home & Living'),(21,'itest_cat_ac1_1775131716876'),(63,'itest_cat_ac1_1775250215166'),(22,'itest_cat_ac2_1775131717374'),(64,'itest_cat_ac2_1775250215888'),(10,'Jewellery'),(2,'Kurtis & Tunics'),(4,'Lehengas'),(9,'Loungewear'),(1,'Sarees');
/*!40000 ALTER TABLE `ProductCategories` ENABLE KEYS */;
UNLOCK TABLES;

--
-- Table structure for table `ProductImages`
--

DROP TABLE IF EXISTS `ProductImages`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!50503 SET character_set_client = utf8mb4 */;
CREATE TABLE `ProductImages` (
  `ImageID` bigint NOT NULL AUTO_INCREMENT,
  `ProductID` bigint NOT NULL,
  `display_order` int NOT NULL DEFAULT '0',
  `url` varchar(2048) CHARACTER SET utf8mb4 COLLATE utf8mb4_general_ci NOT NULL COMMENT 'CDN URL for this image',
  `created_at` timestamp NULL DEFAULT CURRENT_TIMESTAMP,
  PRIMARY KEY (`ImageID`),
  KEY `idx_product_order` (`ProductID`,`display_order`),
  CONSTRAINT `ProductImages_ibfk_1` FOREIGN KEY (`ProductID`) REFERENCES `Products` (`ProductID`)
) ENGINE=InnoDB AUTO_INCREMENT=31 DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_general_ci;
/*!40101 SET character_set_client = @saved_cs_client */;

--
-- Dumping data for table `ProductImages`
--

LOCK TABLES `ProductImages` WRITE;
/*!40000 ALTER TABLE `ProductImages` DISABLE KEYS */;
INSERT INTO `ProductImages` VALUES (1,1,1,'https://pub-0c27e99980dc4e98b13e90c4b24edd19.r2.dev/product/Sarees/1_001_saree_1_1.jpeg','2026-03-07 11:09:06'),(2,1,2,'https://pub-0c27e99980dc4e98b13e90c4b24edd19.r2.dev/product/Sarees/1_001_saree_1_2.jpeg','2026-03-07 11:09:06'),(5,2,0,'https://pub-0c27e99980dc4e98b13e90c4b24edd19.r2.dev/product/Kurtis___Tunics/2_0001_1.jpeg','2026-03-07 23:16:26'),(6,2,1,'https://pub-0c27e99980dc4e98b13e90c4b24edd19.r2.dev/product/Kurtis___Tunics/2_0001_2.jpeg','2026-03-07 23:16:26'),(7,2,2,'https://pub-0c27e99980dc4e98b13e90c4b24edd19.r2.dev/product/Kurtis___Tunics/2_0001_3.jpeg','2026-03-07 23:16:26'),(8,2,3,'https://pub-0c27e99980dc4e98b13e90c4b24edd19.r2.dev/product/Kurtis___Tunics/2_0001_4.jpeg','2026-03-07 23:16:26'),(9,3,0,'https://pub-0c27e99980dc4e98b13e90c4b24edd19.r2.dev/product/Kurtis___Tunics/3_002_1.jpeg','2026-03-07 23:17:52'),(10,3,1,'https://pub-0c27e99980dc4e98b13e90c4b24edd19.r2.dev/product/Kurtis___Tunics/3_002_2.jpeg','2026-03-07 23:17:52'),(11,3,2,'https://pub-0c27e99980dc4e98b13e90c4b24edd19.r2.dev/product/Kurtis___Tunics/3_002_3.jpeg','2026-03-07 23:17:52'),(12,3,3,'https://pub-0c27e99980dc4e98b13e90c4b24edd19.r2.dev/product/Kurtis___Tunics/3_002_4.jpeg','2026-03-07 23:17:52'),(13,4,0,'https://pub-0c27e99980dc4e98b13e90c4b24edd19.r2.dev/product/Kurtis___Tunics/4_003_1.jpeg','2026-03-07 23:19:09'),(14,4,1,'https://pub-0c27e99980dc4e98b13e90c4b24edd19.r2.dev/product/Kurtis___Tunics/4_003_2.jpeg','2026-03-07 23:19:09'),(15,4,2,'https://pub-0c27e99980dc4e98b13e90c4b24edd19.r2.dev/product/Kurtis___Tunics/4_003_3.jpeg','2026-03-07 23:19:09'),(16,5,0,'https://pub-0c27e99980dc4e98b13e90c4b24edd19.r2.dev/product/Kurtis___Tunics/5_004_1.jpeg','2026-03-07 23:20:23'),(17,5,1,'https://pub-0c27e99980dc4e98b13e90c4b24edd19.r2.dev/product/Kurtis___Tunics/5_004_2.jpeg','2026-03-07 23:20:23'),(18,5,2,'https://pub-0c27e99980dc4e98b13e90c4b24edd19.r2.dev/product/Kurtis___Tunics/5_004_3.jpeg','2026-03-07 23:20:23'),(19,6,0,'https://pub-0c27e99980dc4e98b13e90c4b24edd19.r2.dev/product/Kurtis___Tunics/6_005_1.jpeg','2026-03-07 23:21:42'),(20,6,1,'https://pub-0c27e99980dc4e98b13e90c4b24edd19.r2.dev/product/Kurtis___Tunics/6_005_2.jpeg','2026-03-07 23:21:42'),(21,7,0,'https://pub-0c27e99980dc4e98b13e90c4b24edd19.r2.dev/product/Kurtis___Tunics/7_006_1.jpeg','2026-03-07 23:22:36'),(22,7,1,'https://pub-0c27e99980dc4e98b13e90c4b24edd19.r2.dev/product/Kurtis___Tunics/7_006_2.jpeg','2026-03-07 23:22:36'),(23,7,2,'https://pub-0c27e99980dc4e98b13e90c4b24edd19.r2.dev/product/Kurtis___Tunics/7_006_3.jpeg','2026-03-07 23:22:36'),(24,7,3,'https://pub-0c27e99980dc4e98b13e90c4b24edd19.r2.dev/product/Kurtis___Tunics/7_006_4.jpeg','2026-03-07 23:22:36'),(27,1,0,'https://pub-0c27e99980dc4e98b13e90c4b24edd19.r2.dev/product/Sarees/1_001_saree_1_1_47563e66.png','2026-03-16 01:22:16'),(28,8,1,'https://pub-0c27e99980dc4e98b13e90c4b24edd19.r2.dev/product/Sarees/8_0009_2_c734d1bd.jpeg','2026-03-19 21:05:28'),(29,8,0,'https://pub-0c27e99980dc4e98b13e90c4b24edd19.r2.dev/product/Sarees/8_0009_1_29f285e8.jpeg','2026-03-19 21:05:28'),(30,8,2,'https://pub-0c27e99980dc4e98b13e90c4b24edd19.r2.dev/product/Sarees/8_0009_3_ca64213b.jpeg','2026-03-19 21:05:28');
/*!40000 ALTER TABLE `ProductImages` ENABLE KEYS */;
UNLOCK TABLES;

--
-- Table structure for table `ProductMoodMapping`
--

DROP TABLE IF EXISTS `ProductMoodMapping`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!50503 SET character_set_client = utf8mb4 */;
CREATE TABLE `ProductMoodMapping` (
  `ProductID` bigint NOT NULL,
  `MoodID` bigint NOT NULL,
  PRIMARY KEY (`ProductID`,`MoodID`),
  KEY `idx_mood_product` (`MoodID`,`ProductID`),
  CONSTRAINT `ProductMoodMapping_ibfk_1` FOREIGN KEY (`ProductID`) REFERENCES `Products` (`ProductID`) ON DELETE CASCADE,
  CONSTRAINT `ProductMoodMapping_ibfk_2` FOREIGN KEY (`MoodID`) REFERENCES `ProductMoods` (`MoodID`) ON DELETE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_general_ci;
/*!40101 SET character_set_client = @saved_cs_client */;

--
-- Dumping data for table `ProductMoodMapping`
--

LOCK TABLES `ProductMoodMapping` WRITE;
/*!40000 ALTER TABLE `ProductMoodMapping` DISABLE KEYS */;
INSERT INTO `ProductMoodMapping` VALUES (1,10),(2,5),(3,5),(4,5),(5,7),(6,7),(7,4),(8,3);
/*!40000 ALTER TABLE `ProductMoodMapping` ENABLE KEYS */;
UNLOCK TABLES;

--
-- Table structure for table `ProductMoods`
--

DROP TABLE IF EXISTS `ProductMoods`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!50503 SET character_set_client = utf8mb4 */;
CREATE TABLE `ProductMoods` (
  `MoodID` bigint NOT NULL AUTO_INCREMENT,
  `MoodName` varchar(255) CHARACTER SET utf8mb4 COLLATE utf8mb4_general_ci NOT NULL,
  PRIMARY KEY (`MoodID`),
  UNIQUE KEY `uq_mood_name` (`MoodName`)
) ENGINE=InnoDB AUTO_INCREMENT=17 DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_general_ci;
/*!40101 SET character_set_client = @saved_cs_client */;

--
-- Dumping data for table `ProductMoods`
--

LOCK TABLES `ProductMoods` WRITE;
/*!40000 ALTER TABLE `ProductMoods` DISABLE KEYS */;
INSERT INTO `ProductMoods` VALUES (3,'Amar Kolkata'),(11,'Aura of Tussar'),(6,'Cotton Carnival'),(9,'Daily Diva'),(2,'Doodh e Alta'),(12,'Gamcha Fusion Series'),(4,'Kanchi Canvas'),(14,'Kantha Tussar Treasures'),(5,'Khadi Roots'),(10,'Mom chitra'),(13,'Silken Whispers'),(8,'Summer Softs'),(1,'TANT-ER-SAAJ'),(15,'The Jamdani Tussar Series'),(7,'The Weaver\'s Whisper'),(16,'Tussar Embroidered Tales');
/*!40000 ALTER TABLE `ProductMoods` ENABLE KEYS */;
UNLOCK TABLES;

--
-- Table structure for table `ProductStatuses`
--

DROP TABLE IF EXISTS `ProductStatuses`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!50503 SET character_set_client = utf8mb4 */;
CREATE TABLE `ProductStatuses` (
  `id` bigint NOT NULL AUTO_INCREMENT,
  `code` varchar(50) CHARACTER SET utf8mb4 COLLATE utf8mb4_general_ci NOT NULL,
  PRIMARY KEY (`id`),
  UNIQUE KEY `code` (`code`)
) ENGINE=InnoDB AUTO_INCREMENT=4 DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_general_ci;
/*!40101 SET character_set_client = @saved_cs_client */;

--
-- Dumping data for table `ProductStatuses`
--

LOCK TABLES `ProductStatuses` WRITE;
/*!40000 ALTER TABLE `ProductStatuses` DISABLE KEYS */;
INSERT INTO `ProductStatuses` VALUES (2,'active'),(3,'archived'),(1,'draft');
/*!40000 ALTER TABLE `ProductStatuses` ENABLE KEYS */;
UNLOCK TABLES;

--
-- Table structure for table `ProductVariants`
--

DROP TABLE IF EXISTS `ProductVariants`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!50503 SET character_set_client = utf8mb4 */;
CREATE TABLE `ProductVariants` (
  `VariantID` bigint NOT NULL AUTO_INCREMENT,
  `ProductID` bigint NOT NULL,
  `SizeID` bigint DEFAULT NULL,
  `ColorID` bigint DEFAULT NULL,
  `AdditionalPrice` int DEFAULT '0' COMMENT 'Additional price in paise over base product price',
  PRIMARY KEY (`VariantID`),
  UNIQUE KEY `uq_variant_combo` (`ProductID`,`SizeID`,`ColorID`),
  KEY `SizeID` (`SizeID`),
  KEY `ColorID` (`ColorID`),
  CONSTRAINT `ProductVariants_ibfk_1` FOREIGN KEY (`ProductID`) REFERENCES `Products` (`ProductID`),
  CONSTRAINT `ProductVariants_ibfk_2` FOREIGN KEY (`SizeID`) REFERENCES `Sizes` (`SizeID`),
  CONSTRAINT `ProductVariants_ibfk_3` FOREIGN KEY (`ColorID`) REFERENCES `Colors` (`ColorID`)
) ENGINE=InnoDB AUTO_INCREMENT=103 DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_general_ci;
/*!40101 SET character_set_client = @saved_cs_client */;

--
-- Dumping data for table `ProductVariants`
--

LOCK TABLES `ProductVariants` WRITE;
/*!40000 ALTER TABLE `ProductVariants` DISABLE KEYS */;
INSERT INTO `ProductVariants` VALUES (1,1,1,NULL,NULL),(2,2,3,NULL,NULL),(3,2,2,NULL,NULL),(4,2,4,NULL,NULL),(5,2,5,NULL,NULL),(6,2,6,NULL,NULL),(7,3,2,NULL,NULL),(8,3,3,NULL,NULL),(9,3,4,NULL,NULL),(10,3,6,NULL,NULL),(11,4,7,NULL,NULL),(12,4,8,NULL,NULL),(13,5,8,NULL,NULL),(14,6,3,NULL,NULL),(15,6,5,NULL,NULL),(16,7,4,NULL,NULL),(17,7,7,NULL,NULL),(18,8,1,NULL,NULL),(19,8,1,NULL,0),(20,3,2,NULL,0),(21,3,3,NULL,0),(22,3,4,NULL,0),(23,3,6,NULL,0),(24,4,7,NULL,0),(25,4,8,NULL,0),(26,5,8,NULL,0),(27,6,3,NULL,0),(28,6,5,NULL,0),(29,7,4,NULL,0),(30,7,7,NULL,0),(31,14,NULL,NULL,0),(32,15,NULL,NULL,0),(67,57,NULL,NULL,0),(68,58,NULL,NULL,0);
/*!40000 ALTER TABLE `ProductVariants` ENABLE KEYS */;
UNLOCK TABLES;

--
-- Table structure for table `Products`
--

DROP TABLE IF EXISTS `Products`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!50503 SET character_set_client = utf8mb4 */;
CREATE TABLE `Products` (
  `ProductID` bigint NOT NULL AUTO_INCREMENT,
  `sku` varchar(100) CHARACTER SET utf8mb4 COLLATE utf8mb4_general_ci DEFAULT NULL,
  `Name` varchar(255) CHARACTER SET utf8mb4 COLLATE utf8mb4_general_ci NOT NULL,
  `slug` varchar(255) CHARACTER SET utf8mb4 COLLATE utf8mb4_general_ci DEFAULT NULL,
  `Description` text CHARACTER SET utf8mb4 COLLATE utf8mb4_general_ci,
  `price_paise` int NOT NULL COMMENT 'Price in paise (╬ô├▓┬╝Γö£Γöñ╬ô├╢┬ú╬ô├╗├┤╬ô├╢┬╝╬ô├▓┬ÑΓò¼├┤Γö£ΓòóΓö¼├║Γò¼├┤Γö£ΓòóΓö£ΓûÆ╬ô├▓┬╝Γö£Γöñ╬ô├╢┬ú╬ô├▓├│╬ô├╢┬╝Γö£Γòæ╬ô├▓┬╝Γö£Γöñ╬ô├╢┬ú╬ô├▓├╣╬ô├╢┬ú╬ô├╢├▒╬ô├▓┬╝Γö£Γöñ╬ô├╢┬ú╬ô├▓├│╬ô├╢┬╝╬ô├▓┬Ñ╬ô├▓┬╝Γö£Γöñ╬ô├╢┬ú╬ô├╗├┤╬ô├╢┬╝Γö£├ªΓò¼├┤Γö£ΓûôΓö¼Γò¥╬ô├╢┬ú╬ô├╢├▒Γò¼├┤Γö£ΓòóΓö¼├║Γò¼├┤Γö£ΓûôΓö£ΓöéΓò¼├┤Γö£ΓòóΓö¼Γò¥╬ô├╢┬ú╬ô├▓├ªΓò¼├┤Γö£ΓûôΓö¼Γò¥╬ô├╢┬ú╬ô├╢├▒Γò¼├┤Γö£ΓòóΓö¼├║Γò¼├┤Γö£ΓûôΓö£ΓöéΓò¼├┤Γö£ΓòóΓö¼├║Γò¼├┤Γö£ΓòùΓö£├Ñ╬ô├▓┬╝Γö£Γöñ╬ô├╢┬ú╬ô├╗├┤╬ô├╢┬╝╬ô├▓┬ÑΓò¼├┤Γö£ΓòóΓö¼├║Γò¼├┤Γö£ΓòóΓö£ΓûÆ╬ô├▓┬╝Γö£Γöñ╬ô├╢┬ú╬ô├▓├│╬ô├╢┬╝Γö£Γòæ╬ô├▓┬╝Γö£Γöñ╬ô├╢┬ú╬ô├╗├┤╬ô├╢┬ú╬ô├╢├⌐╬ô├▓┬╝Γö£Γöñ╬ô├╢┬ú╬ô├▓├│╬ô├╢┬╝╬ô├▓┬ÑΓò¼├┤Γö£ΓòóΓö¼├║Γò¼├┤Γö£ΓûôΓö£┬¬╬ô├▓┬╝Γö£Γöñ╬ô├╢┬ú╬ô├╗├┤╬ô├╢┬╝╬ô├▓┬ÑΓò¼├┤Γö£ΓòóΓö¼├║Γò¼├┤Γö£ΓòóΓö£ΓûÆ╬ô├▓┬╝Γö£Γöñ╬ô├╢┬ú╬ô├▓├│╬ô├╢┬╝Γö£Γòæ╬ô├▓┬╝Γö£Γöñ╬ô├╢┬ú╬ô├╗├┤╬ô├╢┬ú╬ô├▓├║╬ô├▓┬╝Γö£Γöñ╬ô├╢┬ú╬ô├▓├│╬ô├╢┬╝Γö£Γòæ╬ô├▓┬╝Γö£Γöñ╬ô├╢┬ú╬ô├▓├│╬ô├╢┬ú╬ô├╗├å╬ô├▓┬╝Γö£Γöñ╬ô├╢┬ú╬ô├╗├┤╬ô├╢┬╝╬ô├▓┬ÑΓò¼├┤Γö£ΓòóΓö¼├║Γò¼├┤Γö£ΓòóΓö£ΓûÆ╬ô├▓┬╝Γö£Γöñ╬ô├╢┬ú╬ô├▓├│╬ô├╢┬╝Γö£Γòæ╬ô├▓┬╝Γö£Γöñ╬ô├╢┬ú╬ô├╗├┤╬ô├╢┬ú╬ô├╢├⌐╬ô├▓┬╝Γö£Γöñ╬ô├╢┬ú╬ô├▓├│╬ô├╢┬╝╬ô├▓┬Ñ╬ô├▓┬╝Γö£Γöñ╬ô├╢┬ú╬ô├╗├┤╬ô├╢┬╝Γö£├ª╬ô├▓┬╝Γö£Γöñ╬ô├╢┬ú╬ô├╗├┤╬ô├╢┬╝╬ô├▓┬ÑΓò¼├┤Γö£ΓòóΓö¼├║Γò¼├┤Γö£ΓòóΓö£ΓûÆ╬ô├▓┬╝Γö£Γöñ╬ô├╢┬ú╬ô├▓├│╬ô├╢┬╝Γö£Γòæ╬ô├▓┬╝Γö£Γöñ╬ô├╢┬ú╬ô├▓├╣╬ô├╢┬ú╬ô├╢├▒╬ô├▓┬╝Γö£Γöñ╬ô├╢┬ú╬ô├▓├│╬ô├╢┬╝╬ô├▓┬ÑΓò¼├┤Γö£ΓòóΓö¼├║╬ô├╢┬úΓö¼┬¼Γò¼├┤Γö£ΓûôΓö¼Γò¥╬ô├╢┬ú╬ô├╢├▒Γò¼├┤Γö£ΓòóΓö¼├║Γò¼├┤Γö£ΓòùΓö£ΓöñΓò¼├┤Γö£ΓòóΓö¼Γò¥Γò¼├┤Γö£ΓûôΓö¼├æ╬ô├▓┬╝Γö£Γöñ╬ô├╢┬ú╬ô├▓├│╬ô├╢┬╝Γö£Γòæ╬ô├▓┬╝Γö£Γöñ╬ô├╢┬ú╬ô├▓├│╬ô├╢┬ú╬ô├╗├åΓò¼├┤Γö£ΓûôΓö¼Γò¥╬ô├╢┬ú╬ô├╢├▒Γò¼├┤Γö£ΓòóΓö¼├║Γò¼├┤Γö£ΓûôΓö£ΓöéΓò¼├┤Γö£ΓòóΓö¼Γò¥╬ô├╢┬ú╬ô├▓├ªΓò¼├┤Γö£ΓûôΓö¼Γò¥╬ô├╢┬ú╬ô├╢├▒Γò¼├┤Γö£ΓòóΓö¼├║Γò¼├┤Γö£ΓòùΓö£ΓöñΓò¼├┤Γö£ΓòóΓö¼├║Γò¼├┤Γö£ΓòóΓö£ΓîÉΓò¼├┤Γö£ΓûôΓö¼Γò¥╬ô├╢┬ú╬ô├╢├▒Γò¼├┤Γö£ΓòóΓö¼├║Γò¼├┤Γö£ΓûôΓö£ΓöéΓò¼├┤Γö£ΓòóΓö¼Γò¥Γò¼├┤Γö£ΓûôΓö¼├æ╬ô├▓┬╝Γö£Γöñ╬ô├╢┬ú╬ô├▓├│╬ô├╢┬╝Γö£Γòæ╬ô├▓┬╝Γö£Γöñ╬ô├╢┬ú╬ô├╗├┤╬ô├╢┬úΓö¼┬¼Γò¼├┤Γö£ΓûôΓö¼Γò¥╬ô├╢┬ú╬ô├╢├▒Γò¼├┤Γö£ΓòóΓö¼├║Γò¼├┤Γö£ΓòùΓö£ΓöñΓò¼├┤Γö£ΓòóΓö¼Γò¥Γò¼├┤Γö£ΓûôΓö¼├æ╬ô├▓┬╝Γö£Γöñ╬ô├╢┬ú╬ô├▓├│╬ô├╢┬╝Γö£Γòæ╬ô├▓┬╝Γö£Γöñ╬ô├╢┬ú╬ô├▓├│╬ô├╢┬ú╬ô├╗├åΓò¼├┤Γö£ΓûôΓö¼Γò¥╬ô├╢┬ú╬ô├╢├▒Γò¼├┤Γö£ΓòóΓö¼├║Γò¼├┤Γö£ΓûôΓö£ΓöéΓò¼├┤Γö£ΓòóΓö¼Γò¥╬ô├╢┬ú╬ô├▓├ªΓò¼├┤Γö£ΓûôΓö¼Γò¥╬ô├╢┬ú╬ô├╢├▒Γò¼├┤Γö£ΓòóΓö¼├║Γò¼├┤Γö£ΓòùΓö£ΓöñΓò¼├┤Γö£ΓòóΓö¼├║Γò¼├┤Γö£ΓòóΓö£ΓîÉΓò¼├┤Γö£ΓûôΓö¼Γò¥╬ô├╢┬ú╬ô├╢├▒Γò¼├┤Γö£ΓòóΓö¼├║Γò¼├┤Γö£ΓûôΓö£ΓöéΓò¼├┤Γö£ΓòóΓö¼Γò¥╬ô├╢┬ú╬ô├▓├ªΓò¼├┤Γö£ΓûôΓö¼Γò¥╬ô├╢┬ú╬ô├╢├▒Γò¼├┤Γö£ΓòóΓö¼├║Γò¼├┤Γö£ΓûôΓö£ΓòúΓò¼├┤Γö£ΓòóΓö¼├║╬ô├╢┬úΓö£├ª╬ô├▓┬╝Γö£Γöñ╬ô├╢┬ú╬ô├╗├┤╬ô├╢┬╝╬ô├▓┬ÑΓò¼├┤Γö£ΓòóΓö¼├║Γò¼├┤Γö£ΓòóΓö£ΓûÆ╬ô├▓┬╝Γö£Γöñ╬ô├╢┬ú╬ô├▓├│╬ô├╢┬╝Γö£Γòæ╬ô├▓┬╝Γö£',
  `CategoryID` bigint NOT NULL,
  `fabric` varchar(100) CHARACTER SET utf8mb4 COLLATE utf8mb4_general_ci DEFAULT NULL COMMENT 'Common values: Cotton, Silk, Linen, Chiffon, Georgette, Crepe, Organza, Net, Rayon, Viscose, Satin, Velvet, Brocade, Khadi, Muslin, Tussar Silk, Banarasi Silk, Art Silk, Polyester, Cotton Silk',
  `weave` varchar(100) CHARACTER SET utf8mb4 COLLATE utf8mb4_general_ci DEFAULT NULL,
  `occasion` varchar(100) CHARACTER SET utf8mb4 COLLATE utf8mb4_general_ci DEFAULT NULL,
  `has_blouse_piece` tinyint(1) DEFAULT '1',
  `care_instructions` text CHARACTER SET utf8mb4 COLLATE utf8mb4_general_ci,
  `product_status_id` bigint DEFAULT NULL,
  `created_at` timestamp NULL DEFAULT CURRENT_TIMESTAMP,
  `updated_at` timestamp NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (`ProductID`),
  UNIQUE KEY `sku` (`sku`),
  UNIQUE KEY `slug` (`slug`),
  KEY `CategoryID` (`CategoryID`),
  KEY `idx_sku` (`sku`),
  KEY `idx_slug` (`slug`),
  KEY `idx_product_status` (`product_status_id`),
  KEY `idx_fabric` (`fabric`),
  CONSTRAINT `fk_products_product_status` FOREIGN KEY (`product_status_id`) REFERENCES `ProductStatuses` (`id`),
  CONSTRAINT `Products_ibfk_1` FOREIGN KEY (`CategoryID`) REFERENCES `ProductCategories` (`CategoryID`)
) ENGINE=InnoDB AUTO_INCREMENT=100 DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_general_ci;
/*!40101 SET character_set_client = @saved_cs_client */;

--
-- Dumping data for table `Products`
--

LOCK TABLES `Products` WRITE;
/*!40000 ALTER TABLE `Products` DISABLE KEYS */;
INSERT INTO `Products` VALUES (1,'001_saree_1','saree_1',NULL,'saree_1',899999,1,'Satin','Kanjivaram','Puja',1,'saree_1',2,'2026-03-07 11:09:06','2026-03-07 11:09:06'),(2,'0001','kurti_1',NULL,'kurti_1',79999,2,'Cotton','Plain','Daily Wear',0,NULL,2,'2026-03-07 23:16:25','2026-03-07 23:16:25'),(3,'002','kurti_2',NULL,'kurti_2',79999,2,'Cotton','Plain','Daily Wear',0,NULL,2,'2026-03-07 23:17:51','2026-03-07 23:17:51'),(4,'003','kurti_3',NULL,'kurti_3',79999,2,'Cotton','Plain','Daily Wear',0,NULL,2,'2026-03-07 23:19:08','2026-03-07 23:19:08'),(5,'004','kurti_4',NULL,'kurti_4',79999,2,'Cotton','Plain','Daily Wear',0,NULL,2,'2026-03-07 23:20:23','2026-03-07 23:20:23'),(6,'005','kurti_5',NULL,'kurti_5',79999,2,'Cotton','Plain','Daily Wear',0,NULL,2,'2026-03-07 23:21:41','2026-03-07 23:21:41'),(7,'006','kurti_6',NULL,'kurti_6',79999,2,'Cotton','Handloom','Daily Wear',0,NULL,2,'2026-03-07 23:22:36','2026-03-07 23:22:36'),(8,'0009','saree_2',NULL,'desc saree_2',799900,1,'Net','Kota','Festive',1,NULL,2,'2026-03-19 21:05:27','2026-03-19 21:05:27'),(14,NULL,'AC Product',NULL,NULL,1000,21,NULL,NULL,NULL,NULL,NULL,NULL,'2026-04-02 12:08:37',NULL),(15,NULL,'AC2 Product',NULL,NULL,1000,22,NULL,NULL,NULL,NULL,NULL,NULL,'2026-04-02 12:08:38',NULL),(57,NULL,'AC Product',NULL,NULL,1000,63,NULL,NULL,NULL,NULL,NULL,NULL,'2026-04-03 21:03:36',NULL),(58,NULL,'AC2 Product',NULL,NULL,1000,64,NULL,NULL,NULL,NULL,NULL,NULL,'2026-04-03 21:03:36',NULL);
/*!40000 ALTER TABLE `Products` ENABLE KEYS */;
UNLOCK TABLES;

--
-- Table structure for table `Refunds`
--

DROP TABLE IF EXISTS `Refunds`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!50503 SET character_set_client = utf8mb4 */;
CREATE TABLE `Refunds` (
  `refund_id` bigint NOT NULL AUTO_INCREMENT,
  `order_id` bigint NOT NULL,
  `gateway_refund_id` varchar(100) CHARACTER SET utf8mb4 COLLATE utf8mb4_general_ci NOT NULL COMMENT 'Idempotency: same id returns same refund',
  `amount_paise` int NOT NULL,
  `currency` varchar(3) CHARACTER SET utf8mb4 COLLATE utf8mb4_general_ci DEFAULT 'INR',
  `status` enum('pending','processed','failed','needs_review','client_verified') CHARACTER SET utf8mb4 COLLATE utf8mb4_general_ci DEFAULT 'pending',
  `line_items_refunded` json DEFAULT NULL COMMENT 'Optional: [{order_detail_id, quantity_refunded, amount_paise}]',
  `created_at` timestamp NULL DEFAULT CURRENT_TIMESTAMP,
  PRIMARY KEY (`refund_id`),
  UNIQUE KEY `gateway_refund_id` (`gateway_refund_id`),
  KEY `idx_refunds_order` (`order_id`),
  KEY `idx_refunds_gateway` (`gateway_refund_id`),
  CONSTRAINT `Refunds_ibfk_1` FOREIGN KEY (`order_id`) REFERENCES `Orders` (`OrderID`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_general_ci;
/*!40101 SET character_set_client = @saved_cs_client */;

--
-- Dumping data for table `Refunds`
--

LOCK TABLES `Refunds` WRITE;
/*!40000 ALTER TABLE `Refunds` DISABLE KEYS */;
/*!40000 ALTER TABLE `Refunds` ENABLE KEYS */;
UNLOCK TABLES;

--
-- Table structure for table `Reviews`
--

DROP TABLE IF EXISTS `Reviews`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!50503 SET character_set_client = utf8mb4 */;
CREATE TABLE `Reviews` (
  `ReviewID` bigint NOT NULL AUTO_INCREMENT,
  `ProductID` bigint DEFAULT NULL,
  `UserID` bigint DEFAULT NULL,
  `Rating` tinyint NOT NULL,
  `Comment` text CHARACTER SET utf8mb4 COLLATE utf8mb4_general_ci,
  `review_status` enum('pending','approved','rejected') CHARACTER SET utf8mb4 COLLATE utf8mb4_general_ci DEFAULT 'pending',
  `is_verified_purchase` tinyint(1) DEFAULT '0' COMMENT 'Derived from order history; not user-controlled',
  `created_at` timestamp NULL DEFAULT CURRENT_TIMESTAMP,
  PRIMARY KEY (`ReviewID`),
  UNIQUE KEY `uq_reviews_user_product` (`UserID`,`ProductID`),
  KEY `idx_product_review_status` (`ProductID`,`review_status`),
  KEY `idx_reviews_user` (`UserID`),
  CONSTRAINT `Reviews_ibfk_1` FOREIGN KEY (`ProductID`) REFERENCES `Products` (`ProductID`),
  CONSTRAINT `Reviews_ibfk_2` FOREIGN KEY (`UserID`) REFERENCES `Users` (`UserID`),
  CONSTRAINT `Reviews_chk_1` CHECK ((`Rating` between 1 and 5))
) ENGINE=InnoDB AUTO_INCREMENT=9 DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_general_ci;
/*!40101 SET character_set_client = @saved_cs_client */;

--
-- Dumping data for table `Reviews`
--

LOCK TABLES `Reviews` WRITE;
/*!40000 ALTER TABLE `Reviews` DISABLE KEYS */;
/*!40000 ALTER TABLE `Reviews` ENABLE KEYS */;
UNLOCK TABLES;

--
-- Table structure for table `SecurityAuditLog`
--

DROP TABLE IF EXISTS `SecurityAuditLog`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!50503 SET character_set_client = utf8mb4 */;
CREATE TABLE `SecurityAuditLog` (
  `id` bigint NOT NULL AUTO_INCREMENT,
  `event_type` varchar(100) CHARACTER SET utf8mb4 COLLATE utf8mb4_general_ci NOT NULL COMMENT 'e.g. secrets_rotation, config_reload',
  `details` text CHARACTER SET utf8mb4 COLLATE utf8mb4_general_ci,
  `created_at` timestamp NOT NULL DEFAULT CURRENT_TIMESTAMP,
  PRIMARY KEY (`id`),
  KEY `idx_security_audit_created` (`created_at`)
) ENGINE=InnoDB AUTO_INCREMENT=3 DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_general_ci;
/*!40101 SET character_set_client = @saved_cs_client */;

--
-- Dumping data for table `SecurityAuditLog`
--

LOCK TABLES `SecurityAuditLog` WRITE;
/*!40000 ALTER TABLE `SecurityAuditLog` DISABLE KEYS */;
INSERT INTO `SecurityAuditLog` VALUES (1,'secrets_rotation','integration test','2026-04-02 12:08:24'),(2,'secrets_rotation','integration test','2026-04-03 21:08:11');
/*!40000 ALTER TABLE `SecurityAuditLog` ENABLE KEYS */;
UNLOCK TABLES;

--
-- Table structure for table `Sessions`
--

DROP TABLE IF EXISTS `Sessions`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!50503 SET character_set_client = utf8mb4 */;
CREATE TABLE `Sessions` (
  `session_id` varchar(128) CHARACTER SET utf8mb4 COLLATE utf8mb4_general_ci NOT NULL,
  `user_id` bigint DEFAULT NULL,
  `data` json NOT NULL,
  `ip_address` varchar(45) CHARACTER SET utf8mb4 COLLATE utf8mb4_general_ci DEFAULT NULL,
  `last_activity` timestamp NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  `expires_at` timestamp NOT NULL,
  `created_at` timestamp NULL DEFAULT CURRENT_TIMESTAMP,
  PRIMARY KEY (`session_id`),
  KEY `idx_user` (`user_id`),
  KEY `idx_expires` (`expires_at`),
  CONSTRAINT `Sessions_ibfk_1` FOREIGN KEY (`user_id`) REFERENCES `Users` (`UserID`) ON DELETE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_general_ci;
/*!40101 SET character_set_client = @saved_cs_client */;

--
-- Dumping data for table `Sessions`
--

LOCK TABLES `Sessions` WRITE;
/*!40000 ALTER TABLE `Sessions` DISABLE KEYS */;
/*!40000 ALTER TABLE `Sessions` ENABLE KEYS */;
UNLOCK TABLES;

--
-- Table structure for table `Shipments`
--

DROP TABLE IF EXISTS `Shipments`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!50503 SET character_set_client = utf8mb4 */;
CREATE TABLE `Shipments` (
  `shipment_id` bigint NOT NULL AUTO_INCREMENT,
  `order_id` bigint NOT NULL,
  `shiprocket_order_id` varchar(100) CHARACTER SET utf8mb4 COLLATE utf8mb4_general_ci DEFAULT NULL,
  `awb_code` varchar(100) CHARACTER SET utf8mb4 COLLATE utf8mb4_general_ci DEFAULT NULL,
  `carrier` varchar(100) CHARACTER SET utf8mb4 COLLATE utf8mb4_general_ci DEFAULT NULL,
  `shiprocket_status_id` int DEFAULT NULL COMMENT 'Shiprocket shipment_status_id',
  `shiprocket_status_label` varchar(128) CHARACTER SET utf8mb4 COLLATE utf8mb4_general_ci DEFAULT NULL COMMENT 'Shiprocket status label (API or mapped)',
  `status` enum('pending','awb_assigned','label_generated','manifest_generated','pickup_scheduled','picked_up','in_transit','out_for_delivery','delivered','rto_initiated','rto_delivered','cancelled','lost','delayed','failed') CHARACTER SET utf8mb4 COLLATE utf8mb4_general_ci NOT NULL DEFAULT 'pending',
  `tracking_events` json DEFAULT NULL,
  `created_at` timestamp NULL DEFAULT CURRENT_TIMESTAMP,
  `delivered_at` timestamp NULL DEFAULT NULL,
  PRIMARY KEY (`shipment_id`),
  KEY `idx_order` (`order_id`),
  KEY `idx_awb` (`awb_code`),
  CONSTRAINT `Shipments_ibfk_1` FOREIGN KEY (`order_id`) REFERENCES `Orders` (`OrderID`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_general_ci;
/*!40101 SET character_set_client = @saved_cs_client */;

--
-- Dumping data for table `Shipments`
--

LOCK TABLES `Shipments` WRITE;
/*!40000 ALTER TABLE `Shipments` DISABLE KEYS */;
/*!40000 ALTER TABLE `Shipments` ENABLE KEYS */;
UNLOCK TABLES;

--
-- Table structure for table `ShippingAddresses`
--

DROP TABLE IF EXISTS `ShippingAddresses`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!50503 SET character_set_client = utf8mb4 */;
CREATE TABLE `ShippingAddresses` (
  `ShippingAddressID` bigint NOT NULL AUTO_INCREMENT,
  `UserID` bigint DEFAULT NULL,
  `IsDefault` tinyint(1) NOT NULL DEFAULT '0',
  `Country` varchar(100) CHARACTER SET utf8mb4 COLLATE utf8mb4_general_ci NOT NULL,
  `StateRegion` varchar(100) CHARACTER SET utf8mb4 COLLATE utf8mb4_general_ci NOT NULL,
  `City` varchar(100) CHARACTER SET utf8mb4 COLLATE utf8mb4_general_ci NOT NULL,
  `PostalCode` varchar(20) CHARACTER SET utf8mb4 COLLATE utf8mb4_general_ci NOT NULL,
  `Road` varchar(255) CHARACTER SET utf8mb4 COLLATE utf8mb4_general_ci DEFAULT NULL,
  `ApartmentNoOrName` varchar(255) CHARACTER SET utf8mb4 COLLATE utf8mb4_general_ci DEFAULT NULL,
  `RecipientName` varchar(255) CHARACTER SET utf8mb4 COLLATE utf8mb4_general_ci DEFAULT NULL,
  `PhoneNumber` varchar(32) CHARACTER SET utf8mb4 COLLATE utf8mb4_general_ci DEFAULT NULL,
  PRIMARY KEY (`ShippingAddressID`),
  KEY `idx_shipping_user` (`UserID`),
  KEY `idx_shipping_user_default` (`UserID`,`IsDefault`),
  CONSTRAINT `ShippingAddresses_ibfk_1` FOREIGN KEY (`UserID`) REFERENCES `Users` (`UserID`)
) ENGINE=InnoDB AUTO_INCREMENT=63 DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_general_ci;
/*!40101 SET character_set_client = @saved_cs_client */;

--
-- Dumping data for table `ShippingAddresses`
--

LOCK TABLES `ShippingAddresses` WRITE;
/*!40000 ALTER TABLE `ShippingAddresses` DISABLE KEYS */;
INSERT INTO `ShippingAddresses` VALUES (61,104,0,'India','West Bengal','Kolkata','700084','balia main road','deepshikha house',NULL,NULL),(62,104,1,'India','Karnataka','Bangalore Urban','560103','Devarabisanahalli, Bellandur','1st Floor, Aster East, Block, 2a, Embassy Tech Village Rd','test user','9739097329');
/*!40000 ALTER TABLE `ShippingAddresses` ENABLE KEYS */;
UNLOCK TABLES;

--
-- Table structure for table `ShippingMethods`
--

DROP TABLE IF EXISTS `ShippingMethods`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!50503 SET character_set_client = utf8mb4 */;
CREATE TABLE `ShippingMethods` (
  `MethodID` bigint NOT NULL AUTO_INCREMENT,
  `MethodName` varchar(255) CHARACTER SET utf8mb4 COLLATE utf8mb4_general_ci DEFAULT NULL,
  `Cost` decimal(10,2) DEFAULT NULL,
  `EstimatedDeliveryTime` varchar(255) CHARACTER SET utf8mb4 COLLATE utf8mb4_general_ci DEFAULT NULL,
  PRIMARY KEY (`MethodID`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_general_ci;
/*!40101 SET character_set_client = @saved_cs_client */;

--
-- Dumping data for table `ShippingMethods`
--

LOCK TABLES `ShippingMethods` WRITE;
/*!40000 ALTER TABLE `ShippingMethods` DISABLE KEYS */;
/*!40000 ALTER TABLE `ShippingMethods` ENABLE KEYS */;
UNLOCK TABLES;

--
-- Table structure for table `Sizes`
--

DROP TABLE IF EXISTS `Sizes`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!50503 SET character_set_client = utf8mb4 */;
CREATE TABLE `Sizes` (
  `SizeID` bigint NOT NULL AUTO_INCREMENT,
  `SizeName` varchar(50) CHARACTER SET utf8mb4 COLLATE utf8mb4_general_ci NOT NULL,
  PRIMARY KEY (`SizeID`)
) ENGINE=InnoDB AUTO_INCREMENT=9 DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_general_ci;
/*!40101 SET character_set_client = @saved_cs_client */;

--
-- Dumping data for table `Sizes`
--

LOCK TABLES `Sizes` WRITE;
/*!40000 ALTER TABLE `Sizes` DISABLE KEYS */;
INSERT INTO `Sizes` VALUES (1,'Free Size'),(2,'XS'),(3,'S'),(4,'M'),(5,'L'),(6,'XL'),(7,'XXL'),(8,'3XL');
/*!40000 ALTER TABLE `Sizes` ENABLE KEYS */;
UNLOCK TABLES;

--
-- Table structure for table `Transactions`
--

DROP TABLE IF EXISTS `Transactions`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!50503 SET character_set_client = utf8mb4 */;
CREATE TABLE `Transactions` (
  `TransactionID` bigint NOT NULL AUTO_INCREMENT,
  `UserID` bigint NOT NULL,
  `Amount` decimal(10,2) NOT NULL,
  `TransactionDate` timestamp NOT NULL DEFAULT CURRENT_TIMESTAMP,
  `Type` varchar(50) CHARACTER SET utf8mb4 COLLATE utf8mb4_general_ci NOT NULL,
  PRIMARY KEY (`TransactionID`),
  KEY `UserID` (`UserID`),
  CONSTRAINT `Transactions_ibfk_1` FOREIGN KEY (`UserID`) REFERENCES `Users` (`UserID`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_general_ci;
/*!40101 SET character_set_client = @saved_cs_client */;

--
-- Dumping data for table `Transactions`
--

LOCK TABLES `Transactions` WRITE;
/*!40000 ALTER TABLE `Transactions` DISABLE KEYS */;
/*!40000 ALTER TABLE `Transactions` ENABLE KEYS */;
UNLOCK TABLES;

--
-- Table structure for table `UserActivity`
--

DROP TABLE IF EXISTS `UserActivity`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!50503 SET character_set_client = utf8mb4 */;
CREATE TABLE `UserActivity` (
  `ActivityID` bigint NOT NULL AUTO_INCREMENT,
  `UserID` bigint DEFAULT NULL,
  `ActivityType` varchar(255) CHARACTER SET utf8mb4 COLLATE utf8mb4_general_ci NOT NULL,
  `ActivityTime` timestamp NOT NULL DEFAULT CURRENT_TIMESTAMP,
  `ActivityDetails` text CHARACTER SET utf8mb4 COLLATE utf8mb4_general_ci,
  PRIMARY KEY (`ActivityID`),
  KEY `UserID` (`UserID`),
  CONSTRAINT `UserActivity_ibfk_1` FOREIGN KEY (`UserID`) REFERENCES `Users` (`UserID`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_general_ci;
/*!40101 SET character_set_client = @saved_cs_client */;

--
-- Dumping data for table `UserActivity`
--

LOCK TABLES `UserActivity` WRITE;
/*!40000 ALTER TABLE `UserActivity` DISABLE KEYS */;
/*!40000 ALTER TABLE `UserActivity` ENABLE KEYS */;
UNLOCK TABLES;

--
-- Table structure for table `UserRoles`
--

DROP TABLE IF EXISTS `UserRoles`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!50503 SET character_set_client = utf8mb4 */;
CREATE TABLE `UserRoles` (
  `RoleID` bigint NOT NULL AUTO_INCREMENT,
  `RoleName` varchar(255) CHARACTER SET utf8mb4 COLLATE utf8mb4_general_ci NOT NULL,
  PRIMARY KEY (`RoleID`)
) ENGINE=InnoDB AUTO_INCREMENT=104 DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_general_ci;
/*!40101 SET character_set_client = @saved_cs_client */;

--
-- Dumping data for table `UserRoles`
--

LOCK TABLES `UserRoles` WRITE;
/*!40000 ALTER TABLE `UserRoles` DISABLE KEYS */;
INSERT INTO `UserRoles` VALUES (7,'itest_ac1_1775131716876'),(8,'itest_ac2_1775131717374'),(53,'itest_ac1_1775250215166'),(54,'itest_ac2_1775250215888'),(101,'admin'),(102,'super_admin'),(103,'customer');
/*!40000 ALTER TABLE `UserRoles` ENABLE KEYS */;
UNLOCK TABLES;

--
-- Table structure for table `UserStatuses`
--

DROP TABLE IF EXISTS `UserStatuses`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!50503 SET character_set_client = utf8mb4 */;
CREATE TABLE `UserStatuses` (
  `id` bigint NOT NULL AUTO_INCREMENT,
  `code` varchar(50) CHARACTER SET utf8mb4 COLLATE utf8mb4_general_ci NOT NULL,
  PRIMARY KEY (`id`),
  UNIQUE KEY `code` (`code`)
) ENGINE=InnoDB AUTO_INCREMENT=4 DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_general_ci;
/*!40101 SET character_set_client = @saved_cs_client */;

--
-- Dumping data for table `UserStatuses`
--

LOCK TABLES `UserStatuses` WRITE;
/*!40000 ALTER TABLE `UserStatuses` DISABLE KEYS */;
INSERT INTO `UserStatuses` VALUES (1,'active'),(2,'inactive'),(3,'suspended');
/*!40000 ALTER TABLE `UserStatuses` ENABLE KEYS */;
UNLOCK TABLES;

--
-- Table structure for table `Users`
--

DROP TABLE IF EXISTS `Users`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!50503 SET character_set_client = utf8mb4 */;
CREATE TABLE `Users` (
  `UserID` bigint NOT NULL AUTO_INCREMENT,
  `Username` varchar(255) CHARACTER SET utf8mb4 COLLATE utf8mb4_general_ci NOT NULL,
  `auth_provider` enum('email','google') CHARACTER SET utf8mb4 COLLATE utf8mb4_general_ci NOT NULL DEFAULT 'email' COMMENT 'Login method for this account',
  `password_hash` varchar(255) CHARACTER SET utf8mb4 COLLATE utf8mb4_general_ci DEFAULT NULL COMMENT 'Argon2id hash; NULL for google accounts',
  `google_sub` varchar(255) CHARACTER SET utf8mb4 COLLATE utf8mb4_general_ci DEFAULT NULL COMMENT 'Google subject ID (sub claim); NULL for email accounts',
  `Email` varchar(255) CHARACTER SET utf8mb4 COLLATE utf8mb4_general_ci NOT NULL,
  `email_verified` tinyint(1) DEFAULT '0',
  `email_verified_at` timestamp NULL DEFAULT NULL,
  `FullName` varchar(255) CHARACTER SET utf8mb4 COLLATE utf8mb4_general_ci DEFAULT NULL,
  `Address` text CHARACTER SET utf8mb4 COLLATE utf8mb4_general_ci,
  `Phone` varchar(20) CHARACTER SET utf8mb4 COLLATE utf8mb4_general_ci DEFAULT NULL,
  `user_status_id` bigint DEFAULT NULL,
  `role_id` bigint DEFAULT NULL,
  `last_login_at` timestamp NULL DEFAULT NULL,
  `marketing_opt_out` tinyint(1) DEFAULT '0' COMMENT 'P2: do not send abandoned-cart / marketing emails',
  `CreateDate` timestamp NOT NULL DEFAULT CURRENT_TIMESTAMP,
  `updated_at` timestamp NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (`UserID`),
  UNIQUE KEY `Email` (`Email`),
  UNIQUE KEY `uq_users_username` (`Username`),
  UNIQUE KEY `uq_users_email` (`Email`),
  UNIQUE KEY `uq_users_phone` (`Phone`),
  UNIQUE KEY `uq_users_google_sub` (`google_sub`),
  KEY `idx_email` (`Email`),
  KEY `idx_user_status` (`user_status_id`),
  KEY `idx_user_role` (`role_id`),
  CONSTRAINT `fk_users_role` FOREIGN KEY (`role_id`) REFERENCES `UserRoles` (`RoleID`),
  CONSTRAINT `fk_users_user_status` FOREIGN KEY (`user_status_id`) REFERENCES `UserStatuses` (`id`),
  CONSTRAINT `chk_users_auth_fields` CHECK ((((`auth_provider` = _latin1'email') and (`password_hash` is not null) and (`google_sub` is null)) or ((`auth_provider` = _latin1'google') and (`google_sub` is not null) and (`password_hash` is null))))
) ENGINE=InnoDB AUTO_INCREMENT=130 DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_general_ci;
/*!40101 SET character_set_client = @saved_cs_client */;

--
-- Dumping data for table `Users`
--

LOCK TABLES `Users` WRITE;
/*!40000 ALTER TABLE `Users` DISABLE KEYS */;
INSERT INTO `Users` VALUES (7,'itest_ac1_1775131716876','email','$argon2id$v=19$m=19456,t=2,p=1$YKtN1+XgDWEyXG/cW6E98A$6CYutQKLIgurx6HNk/chIfhrHo0n6qnjsN6w+96cXB0',NULL,'itest_ac1+1775131716876@example.com',0,NULL,NULL,NULL,NULL,NULL,7,NULL,0,'2026-04-02 12:08:37','2026-04-02 12:08:37'),(8,'itest_ac2_1775131717374','email','$argon2id$v=19$m=19456,t=2,p=1$2d7/8ZNq6kFMYD9EOtoB4w$D6VdH6Aq2KjiUArLY8jVXfgF2n+4YOCvzWVRgz6/GV0',NULL,'itest_ac2+1775131717374@example.com',0,NULL,NULL,NULL,NULL,NULL,8,NULL,1,'2026-04-02 12:08:38','2026-04-02 12:08:37'),(54,'itest_ac1_1775250215166','email','$argon2id$v=19$m=19456,t=2,p=1$nPh9BYlBP9QiBZZYG4Uadg$EC6C72Vkaav/LITAR7sNSxFqXYpUIDpWYXV5FrsFoTY',NULL,'itest_ac1+1775250215166@example.com',0,NULL,NULL,NULL,NULL,NULL,53,NULL,0,'2026-04-03 21:03:36','2026-04-03 21:03:35'),(55,'itest_ac2_1775250215888','email','$argon2id$v=19$m=19456,t=2,p=1$wFnlLnT3AlzAdcjM21xl4Q$IHiqwbXtDv16X3xnWPy702srfZ7kMk0T3w1GrMVK9Cw',NULL,'itest_ac2+1775250215888@example.com',0,NULL,NULL,NULL,NULL,NULL,54,NULL,1,'2026-04-03 21:03:36','2026-04-03 21:03:36'),(104,'Sougata Bhattacharjee','google',NULL,'115769956976629038231','sougata.bhattacharjee94@gmail.com',0,NULL,'Sougata Bhattacharjee',NULL,NULL,NULL,NULL,NULL,0,'2026-04-04 22:12:33','2026-04-04 22:12:33'),(106,'Sudatta Bhattacharjee','google',NULL,'112656306409870056151','sudattasdesignerboutique@gmail.com',0,NULL,'Sudatta Bhattacharjee',NULL,NULL,NULL,101,NULL,0,'2026-04-04 22:43:06','2026-04-05 00:28:41');
/*!40000 ALTER TABLE `Users` ENABLE KEYS */;
UNLOCK TABLES;

--
-- Table structure for table `Weaves`
--

DROP TABLE IF EXISTS `Weaves`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!50503 SET character_set_client = utf8mb4 */;
CREATE TABLE `Weaves` (
  `WeaveID` bigint NOT NULL AUTO_INCREMENT,
  `Name` varchar(100) CHARACTER SET utf8mb4 COLLATE utf8mb4_general_ci NOT NULL,
  PRIMARY KEY (`WeaveID`),
  UNIQUE KEY `uq_weave_name` (`Name`)
) ENGINE=InnoDB AUTO_INCREMENT=14 DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_general_ci;
/*!40101 SET character_set_client = @saved_cs_client */;

--
-- Dumping data for table `Weaves`
--

LOCK TABLES `Weaves` WRITE;
/*!40000 ALTER TABLE `Weaves` DISABLE KEYS */;
INSERT INTO `Weaves` VALUES (7,'Banarasi'),(9,'Chanderi'),(5,'Dobby'),(12,'Handloom'),(8,'Ikat'),(4,'Jacquard'),(11,'Jamdani'),(6,'Kanjivaram'),(10,'Kota'),(1,'Plain'),(13,'Powerloom'),(3,'Satin'),(2,'Twill');
/*!40000 ALTER TABLE `Weaves` ENABLE KEYS */;
UNLOCK TABLES;

--
-- Table structure for table `WebhookEvents`
--

DROP TABLE IF EXISTS `WebhookEvents`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!50503 SET character_set_client = utf8mb4 */;
CREATE TABLE `WebhookEvents` (
  `event_id` bigint NOT NULL AUTO_INCREMENT,
  `provider` varchar(50) CHARACTER SET utf8mb4 COLLATE utf8mb4_general_ci NOT NULL,
  `event_type` varchar(100) CHARACTER SET utf8mb4 COLLATE utf8mb4_general_ci NOT NULL,
  `webhook_id` varchar(255) CHARACTER SET utf8mb4 COLLATE utf8mb4_general_ci NOT NULL,
  `provider_event_id` varchar(255) CHARACTER SET utf8mb4 COLLATE utf8mb4_general_ci DEFAULT NULL COMMENT 'e.g. x-razorpay-event-id for replay protection',
  `payload` json NOT NULL,
  `status` enum('pending','processed','failed','needs_review','client_verified') CHARACTER SET utf8mb4 COLLATE utf8mb4_general_ci DEFAULT 'pending',
  `received_at` timestamp NULL DEFAULT CURRENT_TIMESTAMP,
  PRIMARY KEY (`event_id`),
  UNIQUE KEY `uq_webhook_provider_id` (`provider`,`webhook_id`),
  UNIQUE KEY `provider_event_id` (`provider_event_id`),
  KEY `idx_webhook_id` (`webhook_id`),
  KEY `idx_status` (`status`)
) ENGINE=InnoDB AUTO_INCREMENT=25 DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_general_ci;
/*!40101 SET character_set_client = @saved_cs_client */;

--
-- Dumping data for table `WebhookEvents`
--

LOCK TABLES `WebhookEvents` WRITE;
/*!40000 ALTER TABLE `WebhookEvents` DISABLE KEYS */;
INSERT INTO `WebhookEvents` VALUES (15,'shiprocket','shiprocket.tracking','shiprocket:202bb3d0052e346c5aaf81af8a70675c6abb190dd02acc96aec8cc9c653d7eaa',NULL,'{\"awb\": \"123456\", \"etd\": \"2026-04-09 06:10:28\", \"scans\": [{\"date\": \"2022-05-16 16:18:47\", \"status\": \"new\", \"activity\": \"Manifested - Consignment Manifested\", \"location\": \"Mumbai_Chndivli_PC (Maharashtra)\", \"sr-status\": \"NA\", \"sr-status-label\": \"NA\"}, {\"date\": \"2022-05-17 09:59:03\", \"status\": \"assigned_for_seller_pickup\", \"activity\": \"Manifested - Consignment Manifested\", \"location\": \"Mumbai_Chndivli_PC (Maharashtra)\", \"sr-status\": 19, \"sr-status-label\": \"OUT FOR PICKUP\"}], \"order_id\": \"dummpy shiprocket order id 123\", \"is_return\": 0, \"channel_id\": 1234, \"sr_order_id\": 1234, \"courier_name\": \"dummy courier_name\", \"current_status\": \"Delivered\", \"shipment_status\": \"Delivered\", \"current_status_id\": 7, \"current_timestamp\": \"09 04 2026 06:10:28\", \"shipment_status_id\": 7}','processed','2026-04-09 00:40:28'),(16,'shiprocket','shiprocket.tracking','shiprocket:6bc949751ef542bbb7a1f436b43b906229cf64e1ec0ce727b56b7fe758f40a8f',NULL,'{\"awb\": \"123456\", \"etd\": \"2026-04-09 06:10:43\", \"scans\": [{\"date\": \"2022-05-16 16:18:47\", \"status\": \"new\", \"activity\": \"Manifested - Consignment Manifested\", \"location\": \"Mumbai_Chndivli_PC (Maharashtra)\", \"sr-status\": \"NA\", \"sr-status-label\": \"NA\"}, {\"date\": \"2022-05-17 09:59:03\", \"status\": \"assigned_for_seller_pickup\", \"activity\": \"Manifested - Consignment Manifested\", \"location\": \"Mumbai_Chndivli_PC (Maharashtra)\", \"sr-status\": 19, \"sr-status-label\": \"OUT FOR PICKUP\"}], \"order_id\": \"dummpy shiprocket order id 123\", \"is_return\": 0, \"channel_id\": 1234, \"sr_order_id\": 1234, \"courier_name\": \"dummy courier_name\", \"current_status\": \"Delivered\", \"shipment_status\": \"Delivered\", \"current_status_id\": 7, \"current_timestamp\": \"09 04 2026 06:10:43\", \"shipment_status_id\": 7}','processed','2026-04-09 00:40:44'),(17,'shiprocket','shiprocket.tracking','shiprocket:44136fa355b3678a1146ad16f7e8649e94fb4fc21fe77e8310c060f61caaff8a',NULL,'{}','failed','2026-04-09 00:41:05'),(18,'razorpay','payment.authorized','razorpay:pay_SbDvcPTrLpPdS7','SbDw4tEJAOBV3t','{\"event\": \"payment.authorized\", \"entity\": \"event\", \"payload\": {\"payment\": {\"entity\": {\"id\": \"pay_SbDvcPTrLpPdS7\", \"fee\": 0, \"tax\": 0, \"vpa\": null, \"bank\": null, \"card\": {\"id\": \"card_SbDvcPTrLpPdS7\", \"emi\": false, \"iin\": \"999999\", \"name\": \"\", \"type\": \"credit\", \"last4\": \"1005\", \"entity\": \"card\", \"issuer\": \"HDFC\", \"network\": \"RuPay\", \"sub_type\": \"consumer\", \"international\": false}, \"email\": \"void@razorpay.com\", \"notes\": [], \"amount\": 256597, \"entity\": \"payment\", \"method\": \"card\", \"status\": \"authorized\", \"wallet\": null, \"card_id\": \"card_SbDvcPTrLpPdS7\", \"contact\": \"+919739097329\", \"captured\": false, \"currency\": \"INR\", \"order_id\": \"order_SbDvCqtGbJV8YZ\", \"created_at\": 1775700461, \"error_code\": null, \"error_step\": null, \"invoice_id\": null, \"description\": \"Test payment (╬ô├▓┬╝Γö£Γöñ╬ô├╢┬ú╬ô├«├ëΓò¼├┤Γö£ΓûôΓö£Γòæ100)\", \"error_reason\": null, \"error_source\": null, \"acquirer_data\": {\"auth_code\": null, \"authentication_reference_number\": \"\"}, \"international\": false, \"refund_status\": null, \"amount_refunded\": 0, \"error_description\": null, \"amount_transferred\": 0}}}, \"contains\": [\"payment\"], \"account_id\": \"acc_SM1K2oa7Tx1Wcm\", \"created_at\": 1775700487}','processed','2026-04-09 02:08:07'),(19,'razorpay','payment.authorized','razorpay:pay_SbEEdHTKn3IWeH','SbEElhRBF98xEg','{\"event\": \"payment.authorized\", \"entity\": \"event\", \"payload\": {\"payment\": {\"entity\": {\"id\": \"pay_SbEEdHTKn3IWeH\", \"fee\": 0, \"tax\": 0, \"vpa\": null, \"bank\": null, \"card\": {\"id\": \"card_SbEEdHTKn3IWeH\", \"emi\": false, \"iin\": \"999999\", \"name\": \"\", \"type\": \"credit\", \"last4\": \"1005\", \"entity\": \"card\", \"issuer\": \"HDFC\", \"network\": \"RuPay\", \"sub_type\": \"consumer\", \"international\": false}, \"email\": \"void@razorpay.com\", \"notes\": [], \"amount\": 92899, \"entity\": \"payment\", \"method\": \"card\", \"status\": \"authorized\", \"wallet\": null, \"card_id\": \"card_SbEEdHTKn3IWeH\", \"contact\": \"+919739097329\", \"captured\": false, \"currency\": \"INR\", \"order_id\": \"order_SbEEP3gsxofl1k\", \"created_at\": 1775701541, \"error_code\": null, \"error_step\": null, \"invoice_id\": null, \"description\": \"Test payment (╬ô├▓┬╝Γö£Γöñ╬ô├╢┬ú╬ô├«├ëΓò¼├┤Γö£ΓûôΓö£Γòæ100)\", \"error_reason\": null, \"error_source\": null, \"acquirer_data\": {\"auth_code\": null, \"authentication_reference_number\": \"\"}, \"international\": false, \"refund_status\": null, \"amount_refunded\": 0, \"error_description\": null, \"amount_transferred\": 0}}}, \"contains\": [\"payment\"], \"account_id\": \"acc_SM1K2oa7Tx1Wcm\", \"created_at\": 1775701548}','processed','2026-04-09 02:25:49'),(20,'shiprocket','shiprocket.tracking','shiprocket:0ccf193400ee7716ad7b0e8f046ff91d7d8b4b7a4ef757ff1f9639442ded98da',NULL,'{\"awb\": \"123456\", \"etd\": \"2026-04-10 02:55:43\", \"scans\": [{\"date\": \"2022-05-16 16:18:47\", \"status\": \"new\", \"activity\": \"Manifested - Consignment Manifested\", \"location\": \"Mumbai_Chndivli_PC (Maharashtra)\", \"sr-status\": \"NA\", \"sr-status-label\": \"NA\"}, {\"date\": \"2022-05-17 09:59:03\", \"status\": \"assigned_for_seller_pickup\", \"activity\": \"Manifested - Consignment Manifested\", \"location\": \"Mumbai_Chndivli_PC (Maharashtra)\", \"sr-status\": 19, \"sr-status-label\": \"OUT FOR PICKUP\"}], \"order_id\": \"dummpy shiprocket order id 123\", \"is_return\": 0, \"channel_id\": 1234, \"sr_order_id\": 1234, \"courier_name\": \"dummy courier_name\", \"current_status\": \"Delivered\", \"shipment_status\": \"Delivered\", \"current_status_id\": 7, \"current_timestamp\": \"10 04 2026 02:55:43\", \"shipment_status_id\": 7}','processed','2026-04-09 21:25:43'),(21,'shiprocket','shiprocket.tracking','shiprocket:91ad45a9b4dea31973acab47fcfdf95561422cdea805459811b509fa6337d78d',NULL,'{\"awb\": \"123456\", \"etd\": \"2026-04-10 02:55:53\", \"scans\": [{\"date\": \"2022-05-16 16:18:47\", \"status\": \"new\", \"activity\": \"Manifested - Consignment Manifested\", \"location\": \"Mumbai_Chndivli_PC (Maharashtra)\", \"sr-status\": \"NA\", \"sr-status-label\": \"NA\"}, {\"date\": \"2022-05-17 09:59:03\", \"status\": \"assigned_for_seller_pickup\", \"activity\": \"Manifested - Consignment Manifested\", \"location\": \"Mumbai_Chndivli_PC (Maharashtra)\", \"sr-status\": 19, \"sr-status-label\": \"OUT FOR PICKUP\"}], \"order_id\": \"dummpy shiprocket order id 123\", \"is_return\": 0, \"channel_id\": 1234, \"sr_order_id\": 1234, \"courier_name\": \"dummy courier_name\", \"current_status\": \"Delivered\", \"shipment_status\": \"Delivered\", \"current_status_id\": 7, \"current_timestamp\": \"10 04 2026 02:55:53\", \"shipment_status_id\": 7}','processed','2026-04-09 21:25:52'),(22,'shiprocket','shiprocket.tracking','shiprocket:3f5b5a50569703c5a966dac87a186c66c9c3d7807f520b6a6aeb0705f62c9656',NULL,'{\"awb\": \"123456\", \"etd\": \"2026-04-10 02:56:04\", \"scans\": [{\"date\": \"2022-05-16 16:18:47\", \"status\": \"new\", \"activity\": \"Manifested - Consignment Manifested\", \"location\": \"Mumbai_Chndivli_PC (Maharashtra)\", \"sr-status\": \"NA\", \"sr-status-label\": \"NA\"}, {\"date\": \"2022-05-17 09:59:03\", \"status\": \"assigned_for_seller_pickup\", \"activity\": \"Manifested - Consignment Manifested\", \"location\": \"Mumbai_Chndivli_PC (Maharashtra)\", \"sr-status\": 19, \"sr-status-label\": \"OUT FOR PICKUP\"}], \"order_id\": \"dummpy shiprocket order id 123\", \"is_return\": 0, \"channel_id\": 1234, \"sr_order_id\": 1234, \"courier_name\": \"dummy courier_name\", \"current_status\": \"Delivered\", \"shipment_status\": \"Delivered\", \"current_status_id\": 7, \"current_timestamp\": \"10 04 2026 02:56:04\", \"shipment_status_id\": 7}','processed','2026-04-09 21:26:05'),(23,'razorpay','payment.authorized','razorpay:pay_SbXrgPUbpm9IVs','SbXrmSOmabfFLj','{\"event\": \"payment.authorized\", \"entity\": \"event\", \"payload\": {\"payment\": {\"entity\": {\"id\": \"pay_SbXrgPUbpm9IVs\", \"fee\": 0, \"tax\": 0, \"vpa\": null, \"bank\": null, \"card\": {\"id\": \"card_SbXrgPUbpm9IVs\", \"emi\": false, \"iin\": \"999999\", \"name\": \"\", \"type\": \"credit\", \"last4\": \"1005\", \"entity\": \"card\", \"issuer\": \"HDFC\", \"network\": \"RuPay\", \"sub_type\": \"consumer\", \"international\": false}, \"email\": \"void@razorpay.com\", \"notes\": [], \"amount\": 256597, \"entity\": \"payment\", \"method\": \"card\", \"status\": \"authorized\", \"wallet\": null, \"card_id\": \"card_SbXrgPUbpm9IVs\", \"contact\": \"+919739097329\", \"captured\": false, \"currency\": \"INR\", \"order_id\": \"order_SbXr5nH222wmXV\", \"created_at\": 1775770669, \"error_code\": null, \"error_step\": null, \"invoice_id\": null, \"description\": \"Test payment (Γò¼├┤Γö£ΓîÉ╬ô├▓├║100)\", \"error_reason\": null, \"error_source\": null, \"acquirer_data\": {\"auth_code\": null, \"authentication_reference_number\": \"\"}, \"international\": false, \"refund_status\": null, \"amount_refunded\": 0, \"error_description\": null, \"amount_transferred\": 0}}}, \"contains\": [\"payment\"], \"account_id\": \"acc_SM1K2oa7Tx1Wcm\", \"created_at\": 1775770675}','processed','2026-04-09 21:37:55'),(24,'razorpay','payment.authorized','razorpay:pay_SbYKnrmpYGxFkE','SbYKtew34An4vP','{\"event\": \"payment.authorized\", \"entity\": \"event\", \"payload\": {\"payment\": {\"entity\": {\"id\": \"pay_SbYKnrmpYGxFkE\", \"fee\": 0, \"tax\": 0, \"vpa\": null, \"bank\": null, \"card\": {\"id\": \"card_SbYKnrmpYGxFkE\", \"emi\": false, \"iin\": \"999999\", \"name\": \"\", \"type\": \"credit\", \"last4\": \"1005\", \"entity\": \"card\", \"issuer\": \"HDFC\", \"network\": \"RuPay\", \"sub_type\": \"consumer\", \"international\": false}, \"email\": \"void@razorpay.com\", \"notes\": [], \"amount\": 90499, \"entity\": \"payment\", \"method\": \"card\", \"status\": \"authorized\", \"wallet\": null, \"card_id\": \"card_SbYKnrmpYGxFkE\", \"contact\": \"+919739097329\", \"captured\": false, \"currency\": \"INR\", \"order_id\": \"order_SbYKadGowRxsfA\", \"created_at\": 1775772323, \"error_code\": null, \"error_step\": null, \"invoice_id\": null, \"description\": \"Test payment (╬ô├⌐Γòú100)\", \"error_reason\": null, \"error_source\": null, \"acquirer_data\": {\"auth_code\": null, \"authentication_reference_number\": \"\"}, \"international\": false, \"refund_status\": null, \"amount_refunded\": 0, \"error_description\": null, \"amount_transferred\": 0}}}, \"contains\": [\"payment\"], \"account_id\": \"acc_SM1K2oa7Tx1Wcm\", \"created_at\": 1775772329}','processed','2026-04-09 22:05:28');
/*!40000 ALTER TABLE `WebhookEvents` ENABLE KEYS */;
UNLOCK TABLES;

--
-- Table structure for table `Wishlist`
--

DROP TABLE IF EXISTS `Wishlist`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!50503 SET character_set_client = utf8mb4 */;
CREATE TABLE `Wishlist` (
  `WishlistID` bigint NOT NULL AUTO_INCREMENT,
  `UserID` bigint DEFAULT NULL,
  `ProductID` bigint DEFAULT NULL,
  `DateAdded` timestamp NOT NULL DEFAULT CURRENT_TIMESTAMP,
  PRIMARY KEY (`WishlistID`),
  KEY `UserID` (`UserID`),
  KEY `ProductID` (`ProductID`),
  CONSTRAINT `Wishlist_ibfk_1` FOREIGN KEY (`UserID`) REFERENCES `Users` (`UserID`),
  CONSTRAINT `Wishlist_ibfk_2` FOREIGN KEY (`ProductID`) REFERENCES `Products` (`ProductID`)
) ENGINE=InnoDB AUTO_INCREMENT=12 DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_general_ci;
/*!40101 SET character_set_client = @saved_cs_client */;

--
-- Dumping data for table `Wishlist`
--

LOCK TABLES `Wishlist` WRITE;
/*!40000 ALTER TABLE `Wishlist` DISABLE KEYS */;
/*!40000 ALTER TABLE `Wishlist` ENABLE KEYS */;
UNLOCK TABLES;

--
-- Table structure for table `schema_migrations`
--

DROP TABLE IF EXISTS `schema_migrations`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!50503 SET character_set_client = utf8mb4 */;
CREATE TABLE `schema_migrations` (
  `file_name` varchar(255) NOT NULL,
  `applied_at` timestamp NOT NULL DEFAULT CURRENT_TIMESTAMP,
  PRIMARY KEY (`file_name`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci;
/*!40101 SET character_set_client = @saved_cs_client */;

--
-- Dumping data for table `schema_migrations`
--

LOCK TABLES `schema_migrations` WRITE;
/*!40000 ALTER TABLE `schema_migrations` DISABLE KEYS */;
INSERT INTO `schema_migrations` VALUES ('2026-04-04_add_shippingaddresses_isdefault.sql','2026-04-03 19:09:30'),('2026-04-05_seed_admin_roles.sql','2026-04-05 00:05:43'),('2026-04-09_add_shippingaddresses_recipient_name.sql','2026-04-09 01:15:04'),('2026-04-10_add_shippingaddresses_phone_number.sql','2026-04-09 21:52:07'),('20260408_shipments_granular_shiprocket_status.sql','2026-04-08 23:46:25');
/*!40000 ALTER TABLE `schema_migrations` ENABLE KEYS */;
UNLOCK TABLES;
/*!40103 SET TIME_ZONE=@OLD_TIME_ZONE */;

/*!40101 SET SQL_MODE=@OLD_SQL_MODE */;
/*!40014 SET FOREIGN_KEY_CHECKS=@OLD_FOREIGN_KEY_CHECKS */;
/*!40014 SET UNIQUE_CHECKS=@OLD_UNIQUE_CHECKS */;
/*!40101 SET CHARACTER_SET_CLIENT=@OLD_CHARACTER_SET_CLIENT */;
/*!40101 SET CHARACTER_SET_RESULTS=@OLD_CHARACTER_SET_RESULTS */;
/*!40101 SET COLLATION_CONNECTION=@OLD_COLLATION_CONNECTION */;
/*!40111 SET SQL_NOTES=@OLD_SQL_NOTES */;

-- Dump completed on 2026-04-09 23:07:44
