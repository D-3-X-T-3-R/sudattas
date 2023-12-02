SET character_set_client = utf8mb4;

CREATE DATABASE `SUDATTAS`;
USE `SUDATTAS`;

-- Dropping existing tables if they exist;

-- Dropping tables with dependencies first

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
DROP TABLE IF EXISTS `Users`;

/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!40101 SET character_set_client = utf8 */;

-- Table structure for table `Users`
CREATE TABLE `Users` (
    `UserID` bigint NOT NULL AUTO_INCREMENT,
    `Username` varchar(255) NOT NULL,
    `Password` varchar(255) NOT NULL,
    `Email` varchar(255) NOT NULL UNIQUE,
    `FullName` varchar(255) DEFAULT NULL,
    `Address` text,
    `Phone` varchar(20) DEFAULT NULL,
    `CreateDate` timestamp NOT NULL ,
    PRIMARY KEY (`UserID`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_general_ci;

-- Table structure for table `Categories`

CREATE TABLE `Categories` (
    `CategoryID` bigint NOT NULL AUTO_INCREMENT,
    `Name` varchar(255) NOT NULL,
    PRIMARY KEY (`CategoryID`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_general_ci;

-- Table structure for table `Products`

CREATE TABLE `Products` (
    `ProductID` bigint NOT NULL AUTO_INCREMENT,
    `Name` varchar(255) NOT NULL,
    `Description` text,
    `Price` decimal(10,2) NOT NULL,
    `StockQuantity` bigint DEFAULT NULL,
    `CategoryID` bigint DEFAULT NULL,
    PRIMARY KEY (`ProductID`),
    FOREIGN KEY (`CategoryID`) REFERENCES `Categories`(`CategoryID`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_general_ci;

-- Table structure for table `OrderStatus`

CREATE TABLE `OrderStatus` (
    `StatusID` bigint NOT NULL AUTO_INCREMENT,
    `StatusName` varchar(50) NOT NULL,
    PRIMARY KEY (`StatusID`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_general_ci;

-- Table structure for table `Orders`
CREATE TABLE `Orders` (
    `OrderID` bigint NOT NULL AUTO_INCREMENT,
    `UserID` bigint NOT NULL,
    `OrderDate` timestamp NOT NULL,
    `ShippingAddress` text,
    `TotalAmount` decimal(10,2) NOT NULL,
    `StatusID` bigint NOT NULL,
    PRIMARY KEY (`OrderID`),
    FOREIGN KEY (`UserID`) REFERENCES `Users`(`UserID`),
    FOREIGN KEY (`StatusID`) REFERENCES `OrderStatus`(`StatusID`)
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

-- Table structure for table `Reviews`
CREATE TABLE `Reviews` (
    `ReviewID` bigint NOT NULL AUTO_INCREMENT,
    `ProductID` bigint DEFAULT NULL,
    `UserID` bigint DEFAULT NULL,
    `Rating` bigint DEFAULT NULL,
    `Comment` text,
    PRIMARY KEY (`ReviewID`),
    FOREIGN KEY (`ProductID`) REFERENCES `Products`(`ProductID`),
    FOREIGN KEY (`UserID`) REFERENCES `Users`(`UserID`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_general_ci;

-- Table structure for table `Cart`
CREATE TABLE `Cart` (
    `CartID` bigint NOT NULL AUTO_INCREMENT,
    `UserID` bigint NOT NULL,
    `ProductID` bigint NOT NULL,
    `Quantity` bigint NOT NULL,
    PRIMARY KEY (`CartID`),
    FOREIGN KEY (`UserID`) REFERENCES `Users`(`UserID`),
    FOREIGN KEY (`ProductID`) REFERENCES `Products`(`ProductID`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_general_ci;

-- Table structure for table `Wishlist`
CREATE TABLE `Wishlist` (
    `WishlistID` bigint NOT NULL AUTO_INCREMENT,
    `UserID` bigint DEFAULT NULL,
    `ProductID` bigint DEFAULT NULL,
    `DateAdded` timestamp NOT NULL ,
    PRIMARY KEY (`WishlistID`),
    FOREIGN KEY (`UserID`) REFERENCES `Users`(`UserID`),
    FOREIGN KEY (`ProductID`) REFERENCES `Products`(`ProductID`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_general_ci;

-- Table structure for table `ProductImages`
CREATE TABLE `ProductImages` (
    `ImageID` bigint NOT NULL AUTO_INCREMENT,
    `ProductID` bigint DEFAULT NULL,
    `ImageURL` varchar(255) DEFAULT NULL,
    `AltText` varchar(255) DEFAULT NULL,
    PRIMARY KEY (`ImageID`),
    FOREIGN KEY (`ProductID`) REFERENCES `Products`(`ProductID`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_general_ci;

-- Table structure for table `Suppliers`
CREATE TABLE `Suppliers` (
    `SupplierID` bigint NOT NULL AUTO_INCREMENT,
    `Name` varchar(255) DEFAULT NULL,
    `ContactInfo` varchar(255) DEFAULT NULL,
    `Address` text,
    PRIMARY KEY (`SupplierID`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_general_ci;

-- Table structure for table `Inventory`
CREATE TABLE `Inventory` (
    `InventoryID` bigint NOT NULL AUTO_INCREMENT,
    `ProductID` bigint DEFAULT NULL,
    `QuantityAvailable` bigint DEFAULT NULL,
    `ReorderLevel` bigint DEFAULT NULL,
    `SupplierID` bigint DEFAULT NULL,
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
    `TransactionDate` timestamp NOT NULL ,
    `Type` varchar(50) NOT NULL,
    PRIMARY KEY (`TransactionID`),
    FOREIGN KEY (`UserID`) REFERENCES `Users`(`UserID`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_general_ci;

-- Table structure for table `NewsletterSubscribers`
CREATE TABLE `NewsletterSubscribers` (
    `SubscriberID` bigint NOT NULL AUTO_INCREMENT,
    `Email` varchar(255) NOT NULL UNIQUE,
    `SubscriptionDate` timestamp NOT NULL ,
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

-- ============================================================================================

;

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

-- ===========================================================================================

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

CREATE TABLE `EventLogs` (
    `LogID` bigint NOT NULL AUTO_INCREMENT,
    `EventType` varchar(255) NOT NULL,
    `EventDescription` text,
    `UserID` bigint DEFAULT NULL,
    `EventTime` timestamp NOT NULL ,
    PRIMARY KEY (`LogID`),
    FOREIGN KEY (`UserID`) REFERENCES `Users`(`UserID`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_general_ci;

CREATE TABLE `UserActivity` (
    `ActivityID` bigint NOT NULL AUTO_INCREMENT,
    `UserID` bigint DEFAULT NULL,
    `ActivityType` varchar(255) NOT NULL,
    `ActivityTime` timestamp NOT NULL ,
    `ActivityDetails` text,
    PRIMARY KEY (`ActivityID`),
    FOREIGN KEY (`UserID`) REFERENCES `Users`(`UserID`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_general_ci;

CREATE TABLE `InventoryLog` (
    `LogID` bigint NOT NULL AUTO_INCREMENT,
    `ProductID` bigint NOT NULL,
    `ChangeQuantity` bigint NOT NULL,
    `LogTime` timestamp NOT NULL ,
    `Reason` varchar(255) DEFAULT NULL,
    PRIMARY KEY (`LogID`),
    FOREIGN KEY (`ProductID`) REFERENCES `Products`(`ProductID`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_general_ci;

CREATE TABLE `Promotions` (
    `PromotionID` bigint NOT NULL AUTO_INCREMENT,
    `PromotionName` varchar(255) NOT NULL,
    `StartDate` datetime NOT NULL,
    `EndDate` datetime NOT NULL,
    `Details` text,
    PRIMARY KEY (`PromotionID`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_general_ci;

CREATE TABLE `ShippingZones` (
    `ZoneID` bigint NOT NULL AUTO_INCREMENT,
    `ZoneName` varchar(255) NOT NULL,
    `Description` text,
    PRIMARY KEY (`ZoneID`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_general_ci;

CREATE TABLE `PaymentMethods` (
    `MethodID` bigint NOT NULL AUTO_INCREMENT,
    `MethodName` varchar(255) NOT NULL,
    `Details` text,
    PRIMARY KEY (`MethodID`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_general_ci;

;
