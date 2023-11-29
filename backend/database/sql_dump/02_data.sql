-- Inserting data into `Categories`
USE `SUDATTAS`;

INSERT INTO `Categories` (`Name`)
VALUES
('Electronics'),
('Books'),
('Clothing');

-- Inserting data into `Sizes`
INSERT INTO `Sizes` (`SizeName`)
VALUES
('Small'),
('Medium'),
('Large');

-- Inserting data into `Colors`
INSERT INTO `Colors` (`ColorName`)
VALUES
('Red'),
('Blue'),
('Green');

-- Inserting data into `UserRoles`
INSERT INTO `UserRoles` (`RoleName`)
VALUES
('Admin'),
('Customer');

-- Inserting data into `ShippingMethods`
INSERT INTO `ShippingMethods` (`MethodName`, `Cost`, `EstimatedDeliveryTime`)
VALUES
('Standard Shipping', 4.99, '3-5 Business Days'),
('Express Shipping', 19.99, '1-2 Business Days');

-- Inserting data into `Suppliers`
INSERT INTO `Suppliers` (`Name`, `ContactInfo`, `Address`)
VALUES
('Gadgets Inc', 'contact@gadgetsinc.com', '789 Electronics Blvd'),
('Books Store', 'info@booksstore.com', '321 Literary Lane');

-- Inserting data into `NewsletterSubscribers`
INSERT INTO `NewsletterSubscribers` (`Email`, `SubscriptionDate`)
VALUES
('subscriber@example.com', CURRENT_TIMESTAMP),
('user@example.com', CURRENT_TIMESTAMP);

-- Inserting data into `ShippingZones`
INSERT INTO `ShippingZones` (`ZoneName`, `Description`)
VALUES
('Zone 1', 'Local zone'),
('Zone 2', 'International zone');

-- Inserting data into `PaymentMethods`
INSERT INTO `PaymentMethods` (`MethodName`, `Details`)
VALUES
('Credit Card', 'Pay with credit card'),
('PayPal', 'Pay via PayPal');

-- Inserting data into `Promotions`
INSERT INTO `Promotions` (`PromotionName`, `StartDate`, `EndDate`, `Details`)
VALUES
('Summer Sale', '2023-06-01 00:00:00', '2023-06-30 23:59:59', 'Discounts on all clothing items');

-- Now that the tables without foreign keys have data, we can insert data into tables with foreign keys.

-- Inserting data into `Users`
INSERT INTO `Users` (`Username`, `Password`, `Email`, `FullName`, `Address`, `Phone`, `CreateDate`)
VALUES
('john_doe', 'hashed_password', 'john.doe@example.com', 'John Doe', '123 Maple Street', '555-1234', CURRENT_TIMESTAMP),
('jane_doe', 'hashed_password', 'jane.doe@example.com', 'Jane Doe', '456 Oak Avenue', '555-5678', CURRENT_TIMESTAMP);

-- Proceed with the `Products` table, which depends on `Categories`:
INSERT INTO `Products` (`Name`, `Description`, `Price`, `StockQuantity`, `CategoryID`)
VALUES
('Smartphone', 'Latest model smartphone with high specs.', 799.99, 50, 1),
('Novel Book', 'A compelling new novel by a bestselling author.', 19.99, 100, 2),
('T-Shirt', 'Comfortable cotton t-shirt.', 9.99, 150, 3);

-- ==================================================================================================

-- Now that Users and Categories have data, we can insert data into Products, which depends on Categories.
-- Inserting data into `Products`
INSERT INTO `Products` (`Name`, `Description`, `Price`, `StockQuantity`, `CategoryID`)
VALUES
('Smartphone', 'Latest model smartphone with high specs.', 799.99, 50, 1),
('Novel Book', 'A compelling new novel by a bestselling author.', 19.99, 100, 2),
('T-Shirt', 'Comfortable cotton t-shirt.', 9.99, 150, 3);

-- Assuming that we have inserted data into Products and we have `ProductID` values (1, 2, 3, ...)
-- Inserting data into `Orders`
INSERT INTO `Orders` (`UserID`, `OrderDate`, `ShippingAddress`, `TotalAmount`, `Status`)
VALUES
(1, CURRENT_TIMESTAMP, '123 Maple Street', 829.98, 'Processing'),
(2, CURRENT_TIMESTAMP, '456 Oak Avenue', 29.98, 'Shipped');

-- Assuming that we have inserted data into Orders and we have `OrderID` values (1, 2, ...)
-- Inserting data into `OrderDetails`
INSERT INTO `OrderDetails` (`OrderID`, `ProductID`, `Quantity`, `Price`)
VALUES
(1, 1, 1, 799.99),
(2, 2, 1, 19.99);

-- Assuming that we have inserted data into Products and we have `ProductID` values (1, 2, ...)
-- Inserting data into `Reviews`
INSERT INTO `Reviews` (`ProductID`, `UserID`, `Rating`, `Comment`)
VALUES
(1, 1, 5, 'Excellent phone with amazing features.'),
(2, 2, 4, 'Really enjoyed the book.');

-- Assuming that we have inserted data into Users and Products and we have `UserID` and `ProductID` values (1, 2, ...)
-- Inserting data into `Cart`
INSERT INTO `Cart` (`UserID`, `ProductID`, `Quantity`)
VALUES
(1, 3, 2),
(2, 1, 1);

-- Inserting data into `Wishlist`
INSERT INTO `Wishlist` (`UserID`, `ProductID`, `DateAdded`)
VALUES
(1, 2, CURRENT_TIMESTAMP),
(2, 3, CURRENT_TIMESTAMP);

-- Inserting data into `ProductImages`
INSERT INTO `ProductImages` (`ProductID`, `ImageURL`, `AltText`)
VALUES
(1, 'http://example.com/images/smartphone.jpg', 'Smartphone Image'),
(2, 'http://example.com/images/novel.jpg', 'Novel Book Image');

-- Assuming that we have inserted data into Products and Suppliers and we have `ProductID` and `SupplierID` values (1, 2, ...)
-- Inserting data into `Inventory`
INSERT INTO `Inventory` (`ProductID`, `QuantityAvailable`, `ReorderLevel`, `SupplierID`)
VALUES
(1, 50, 10, 1),
(2, 100, 20, 2);

-- Assuming that we have inserted data into Products and we have `ProductID` values (1, 2, ...)
-- Inserting data into `ProductAttributes`
INSERT INTO `ProductAttributes` (`ProductID`, `AttributeName`, `AttributeValue`)
VALUES
(1, 'Color', 'Black'),
(1, 'Storage', '128GB');

-- Inserting data into `Discounts`
INSERT INTO `Discounts` (`ProductID`, `DiscountPercentage`, `StartDate`, `EndDate`)
VALUES
(3, 10.00, '2023-01-01', '2023-01-31');

-- Assuming that we have inserted data into Users and UserRoles and we have `UserID` and `RoleID` values (1, 2, ...)
-- Inserting data into `UserRolesMapping`
INSERT INTO `UserRolesMapping` (`UserID`, `RoleID`)
VALUES
(1, 1),
(2, 2);

-- Assuming that we have inserted data into Products and we have `ProductID` values (1, 2, ...)
-- Inserting data into `ProductCategoryMapping`
INSERT INTO `ProductCategoryMapping` (`ProductID`, `CategoryID`)
VALUES
(1, 1),
(2, 2),
(3, 3);

-- Assuming that we have inserted data into Products and ProductAttributes and we have `ProductID` and `AttributeID` values (1, 2, ...)
-- Inserting data into `ProductAttributeMapping`
INSERT INTO `ProductAttributeMapping` (`ProductID`, `AttributeID`)
VALUES
(1, 1),
(2, 2);

-- Assuming that we have inserted data into Users and UserRoles and we have `UserID` and `RoleID` values (1, 2, ...)
-- Inserting data into `UserRoleMapping`
INSERT INTO `UserRoleMapping` (`UserID`, `RoleID`)
VALUES
(1, 1),
(2, 2);

-- Assuming that we have inserted data into Products and Sizes and we have `ProductID` and `SizeID` values (1, 2, ...)
-- Inserting data into `ProductSizeMapping`
INSERT INTO `ProductSizeMapping` (`ProductID`, `SizeID`)
VALUES
(3, 1),
(3, 2),
(3, 3);

-- Assuming that we have inserted data into Products and Colors and we have `ProductID` and `ColorID` values (1, 2, ...)
-- Inserting data into `ProductColorMapping`
INSERT INTO `ProductColorMapping` (`ProductID`, `ColorID`)
VALUES
(3, 1),
(3, 2),
(3, 3);

-- Assuming that we have inserted data into Products, Sizes, and Colors and we have `ProductID`, `SizeID`, and `ColorID` values (1, 2, ...)
-- Inserting data into `ProductVariants`
INSERT INTO `ProductVariants` (`ProductID`, `SizeID`, `ColorID`, `AdditionalPrice`)
VALUES
(3, 1, 1, 0.00),
(3, 2, 2, 0.00),
(3, 3, 3, 0.00);

-- Assuming that we have inserted data into Users and we have `UserID` values (1, 2, ...)
-- Inserting data into `Transactions`
INSERT INTO `Transactions` (`UserID`, `Amount`, `TransactionDate`, `Type`)
VALUES
(1, 799.99, CURRENT_TIMESTAMP, 'Purchase'),
(2, 19.99, CURRENT_TIMESTAMP, 'Purchase');

-- Assuming that we have inserted data into Users and Products and we have `UserID` and `ProductID` values (1, 2, ...)
-- Inserting data into `ProductRatings`
INSERT INTO `ProductRatings` (`ProductID`, `UserID`, `Rating`)
VALUES
(1, 1, 5),
(2, 2, 4);
