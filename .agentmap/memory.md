# Memory

This file contains extracted knowledge markers from the codebase.

## Summary

| Category | Count | High | Medium | Low |
| -------- | ----- | ---- | ------ | --- |
| ⚠️ Warnings | 11 | 11 | 0 | 0 |
| 📋 Business Rules | 4 | 4 | 0 | 0 |
| 🔧 Technical Debt | 12 | 0 | 12 | 0 |
| 📝 Notes | 2 | 0 | 0 | 2 |

---

## ⚠️ Warnings

### 🔴 `WARNING` (OrderController.cs:16)

> This endpoint modifies order state

### 🔴 `SAFETY` (OrderController.cs:38)

> Must validate before processing payment

### 🔴 `WARNING` (OrderService.java:19)

> This method modifies the database directly

### 🔴 `WARNING` (example.c:27)

> Caller must free the returned memory

### 🔴 `WARNING` (example.cpp:35)

> Performance critical

### 🔴 `WARNING` (example.dart:31)

> This is not thread-safe

### 🔴 `WARNING` (example.rb:12)

> Validate all params

### 🔴 `WARNING` (example.swift:22)

> This is a singleton, be careful with state

### 🔴 `WARNING` (order.php:2)

> This file handles payment processing - be careful with changes

### 🔴 `SAFETY` (order.php:38)

> Must check inventory before payment

### 🔴 `WARNING` (order.php:61)

> Deleting orders affects financial reports

---

## 📋 Business Rules

### 🔴 `RULE` (OrderController.cs:25)

> Only order owner can view their orders

### 🔴 `RULE` (OrderService.java:33)

> Orders must have at least one item

### 🔴 `RULE` (order.php:3)

> All order modifications must go through validateOrder() first

### 🔴 `RULE` (order.php:90)

> Refunds require manager approval for amounts > $500

---

## 🔧 Technical Debt

### 🟡 `TODO` (OrderController.cs:6)

> Add authentication middleware

### 🟡 `FIXME` (OrderController.cs:30)

> Add pagination

### 🟡 `TODO` (OrderService.java:8)

> Add caching layer for performance

### 🟡 `FIXME` (OrderService.java:26)

> Needs transaction support

### 🟡 `TODO` (example.c:4)

> Add error handling

### 🟡 `TODO` (example.cpp:4)

> Implement caching

### 🟡 `TODO` (example.dart:3)

> Add proper state management

### 🟡 `TODO` (example.rb:1)

> Add authentication

### 🟡 `TODO` (example.swift:3)

> Add proper error handling

### 🟡 `TODO` (order.php:12)

> Split this file into smaller controllers

### 🟡 `FIXME` (order.php:23)

> This method is too long, needs refactoring

### 🟡 `TODO` (order.php:54)

> Add permission check here

---

## 📝 Notes

### 🟢 `NOTE` (OrderService.java:43)

> Used for admin reporting only

### 🟢 `NOTE` (order.php:82)

> This is used for reporting only

---

