# Code Outline

This file contains symbol maps for large files in the codebase.

## Table of Contents

- [OrderController.cs](#ordercontroller-cs) (58 lines, 16 symbols)
- [OrderService.java](#orderservice-java) (60 lines, 12 symbols)
- [example.c](#example-c) (35 lines, 6 symbols)
- [example.cpp](#example-cpp) (62 lines, 7 symbols)
- [example.dart](#example-dart) (50 lines, 6 symbols)
- [example.rb](#example-rb) (44 lines, 11 symbols)
- [order.php](#order-php) (100 lines, 10 symbols)

---

## OrderController.cs (58 lines)

| Line | Kind | Name | Visibility |
| ---- | ---- | ---- | ---------- |
| 3 | mod | MyApp.Controllers | pub |
| 7 | class | OrderController | pub |
| 10 | method | OrderController | (private) |
| 17 | method | CreateOrder | pub |
| 20 | method | Ok | (private) |
| 22 | method | GetOrder | pub |
| 27 | method | Ok | (private) |
| 31 | method | GetAllOrders | pub |
| 35 | method | ValidateOrder | (private) |
| 42 | interface | IOrderService | pub |
| 45 | method | Create | (private) |
| 46 | method | GetById | (private) |
| 47 | method | GetAll | (private) |
| 49 | struct | Order | pub |
| 49 | method | Order | pub |
| 51 | enum | OrderStatus | pub |

### Key Entry Points

- `public class OrderController` (L7)
- `public record Order` (L49)

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

## example.c (35 lines)

| Line | Kind | Name | Visibility |
| ---- | ---- | ---- | ---------- |
| 5 | struct | anonymous | pub |
| 12 | enum | Color | pub |
| 18 | fn | helper_function | (private) |
| 22 | fn | add | pub |
| 26 | fn | allocate_memory | pub |
| 31 | fn | main | pub |

### Key Entry Points

- `struct anonymous` (L5)
- `int add(int a, int b)` (L22)
- `void* allocate_memory(size_t size)` (L26)
- `int main(int argc, char* argv[])` (L31)

---

## example.cpp (62 lines)

| Line | Kind | Name | Visibility |
| ---- | ---- | ---- | ---------- |
| 5 | mod | game | pub |
| 7 | enum | Color | pub |
| 13 | struct | Point | pub |
| 18 | class | Entity | pub |
| 30 | class | Player | pub |
| 45 | class | Container | pub |
| 58 | fn | main | pub |

### Key Entry Points

- `struct Point` (L13)
- `class Entity` (L18)
- `class Player` (L30)
- `class Container` (L45)
- `int main()` (L58)

---

## example.dart (50 lines)

| Line | Kind | Name | Visibility |
| ---- | ---- | ---- | ---------- |
| 4 | class | BaseWidget | pub |
| 8 | trait | LoggerMixin | pub |
| 14 | mod | StringExtensions | pub |
| 21 | enum | OrderStatus | pub |
| 28 | class | OrderController | pub |
| 45 | class | _PrivateWidget | (private) |

### Key Entry Points

- `abstract class BaseWidget` (L4)
- `class OrderController` (L28)

---

## example.rb (44 lines)

| Line | Kind | Name | Visibility |
| ---- | ---- | ---- | ---------- |
| 2 | mod | Shop | pub |
| 4 | const | order_service | pub |
| 5 | const | logger | pub |
| 5 | const | config | pub |
| 6 | method | initialize | pub |
| 10 | method | create | pub |
| 16 | method | show | pub |
| 21 | fn | self.routes | pub |
| 27 | method | validate_params | (private) |
| 33 | method | log_action | (internal) |
| 40 | method | format_currency | pub |

### Key Entry Points

- `def self.routes` (L21)

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

