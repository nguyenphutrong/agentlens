# Code Outline

This file contains symbol maps for large files in the codebase.

## Table of Contents

- [OrderService.java](#orderservice-java) (60 lines, 12 symbols)
- [order.php](#order-php) (100 lines, 10 symbols)

---

## OrderService.java (60 lines)

| Line | Kind | Name | Visibility |
| ---- | ---- | ---- | ---------- |
| 10 | class | OrderService | pub |
| 13 | method | OrderService | (internal) |
| 20 | method | createOrder | pub |
| 27 | method | processPayment | pub |
| 31 | method | validateRequest | (private) |
| 38 | method | findById | pub |
| 44 | method | findAll | pub |
| 48 | interface | OrderRepository | (internal) |
| 50 | method | save | (internal) |
| 51 | method | findById | (internal) |
| 52 | method | findAll | (internal) |
| 54 | enum | OrderStatus | (internal) |

### Key Entry Points

- `public class OrderService` (L10)

---

## order.php (100 lines)

| Line | Kind | Name | Visibility |
| ---- | ---- | ---- | ---------- |
| 14 | class | OrderController | pub |
| 17 | fn | __construct | pub |
| 24 | fn | createOrder | pub |
| 40 | fn | validateOrder | pub |
| 50 | fn | updateOrder | pub |
| 58 | fn | deleteOrder | pub |
| 66 | fn | checkInventory | pub |
| 76 | fn | validatePayment | pub |
| 83 | fn | getOrderStats | pub |
| 87 | fn | processRefund | pub |

### Key Entry Points

- `class OrderController` (L14)
- `public function __construct(...)` (L17)
- `public function createOrder(...)` (L24)
- `private function validateOrder(...)` (L40)
- `public function updateOrder(...)` (L50)

---

