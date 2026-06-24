# CLAUDE.md

  This file provides guidance to Claude Code (claude.ai/code) when working with this codebase.

  ## Overview
  This is a Rust backend service built with a layered architecture centered around **domain modules + SeaORM
  entities**, backed by a relational database (MySQL in current migration history). The project includes:

  - Authentication system (JWT + WeChat login integration)
  - Core business domain around bookings and scheduling
  - SeaORM-based persistence layer
  - Structured separation between `domain/` (business logic) and `entity/` (database models)

  The entrypoint is `src/main.rs`, which wires application state, routing, and infrastructure together.

  ---

  ## Common Development Commands

  ### Build
  ```bash
  cargo build

  Run (debug)

  cargo run

  Run (release)

  cargo run --release

  Check (fast compile validation)

  cargo check

  Run tests

  cargo test

  Run a single test

  cargo test <test_name>

  Run tests with output

  cargo test -- --nocapture

  Format code

  cargo fmt

  Lint (Clippy)

  cargo clippy
  ```

  ---
  Architecture Overview

  High-Level Structure

  The system is organized into three main layers:

  1. Application Entry (src/main.rs)

  - Initializes configuration (env, database, logging)
  - Builds shared application state
  - Registers HTTP routes / services
  - Bootstraps authentication and middleware

  This is the orchestration layer and should remain thin.

  ---
  2. Domain Layer (src/domain/)

  Business logic is organized by feature:

  - bookings/ – booking lifecycle, creation, validation, status handling
  - booking_logs/ – audit/history tracking for booking actions
  - date_settings/ – configuration of available booking dates
  - date_slots/ – computed or persisted availability slots
  - time_slot_templates/ – reusable scheduling templates

  Key idea:
  Domain modules encapsulate rules and workflows and should NOT depend directly on HTTP or database details. They
  typically interact with entities through service/repository boundaries.

  ---
  3. Persistence Layer (src/entity/)

  SeaORM entities representing database tables:

  - booking
  - booking_log
  - date_setting
  - date_slot
  - time_slot_template

  These are auto-mapped ORM models used for DB operations.

  ---
  Data & ORM Model

  - Uses SeaORM-style entity definitions
  - Entities represent raw persistence structure
  - Domain layer builds business meaning on top of them
  - Likely MySQL schema based on migration history

  When working in this layer:
  - Avoid embedding business logic in entities
  - Keep entities as “dumb data models”

  ---
  Authentication & Security

  The system includes:

  - JWT-based authentication (token issuance + validation)
  - WeChat login integration (OAuth-style flow)
  - Likely middleware-based request authentication

  When modifying auth:
  - Ensure token validation remains centralized
  - Avoid duplicating auth logic in domain modules

  ---
  Booking System Concept (Core Domain)

  The central business model revolves around scheduling:

  - Date Settings define availability rules
  - Time Slot Templates define reusable time structures
  - Date Slots represent concrete availability
  - Bookings reserve slots
  - Booking Logs track state transitions and history

  When modifying booking logic:
  - Ensure consistency between slot availability and booking creation
  - Always update logs for state transitions
  - Treat booking creation as a multi-step transactional workflow

  ---
  Development Notes

  Database Changes

  If modifying entities:
  - Update SeaORM entities in src/entity/
  - Ensure corresponding domain logic in src/domain/ stays consistent

  Debugging Tips

  - Start from main.rs to trace request flow
  - Follow domain module → entity usage chain
  - Check booking-related logic first for most business features

  ---
  Suggested Workflow

  1. Identify feature area (domain module)
  2. Trace entity usage if persistence is involved
  3. Modify domain logic first
  4. Adjust entity mappings only if schema changes are required
  5. Validate via cargo test + manual run

  ---
  Build Mental Model

  Think of the system as:

  ▎ HTTP Layer → Domain Logic → SeaORM Entities → Database

  Most changes should stay in the domain layer, with entities acting as a strict persistence boundary.

  ---