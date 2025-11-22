
# 🚀 Distributed Job Scheduler Platform (Rust + Next.js MVP)

This project is a simplified, high-performance distributed task management system, inspired by tools like Celery and Airflow, built to manage and execute asynchronous background jobs with real-time monitoring.

## ✨ Features

* **RESTful API:** Create and monitor jobs via HTTP endpoints.
* **Asynchronous Worker:** Background workers built with **Rust** and **Tokio** process jobs concurrently.
* **Message Queue:** Uses **Redis** as a reliable, high-speed message queue (`BLPOP/RPUSH` pattern).
* **Data Persistence:** Stores job metadata and results in **PostgreSQL** using the asynchronous **SQLx** crate.
* **Real-time Monitoring:** WebSocket server for instant job status updates to the frontend using **Tokio broadcast channels**.
* **Admin Dashboard:** Modern **Next.js 14** frontend with real-time data visualization via WebSockets.
* **Containerized:** Full multi-service stack orchestrated via **Docker Compose**.

## 🛠️ Technical Stack

| Component | Technology | Role |
| :--- | :--- | :--- |
| **Backend** | **Rust** (Tokio, Axum) | High-performance API and concurrent worker system. |
| **Database** | **PostgreSQL** (SQLx) | Durable storage for job persistence and history. |
| **Messaging** | **Redis** (deadpool-redis) | Message queue for distributing job IDs to workers. |
| **Frontend** | **Next.js 14** (App Router) | Admin dashboard with TypeScript and TailwindCSS. |
| **Real-time** | **WebSockets** (Axum, Tokio Broadcast) | Live status updates without client polling. |
| **Orchestration** | **Docker & Docker Compose** | Defining and running all services. |

## 📦 Getting Started

These instructions will get the project stack up and running on your local machine using Docker Compose.

### Prerequisites

* **Docker** and **Docker Compose** installed.
* **Node.js** and **npm** (for initial frontend setup).

### 1. Initial Setup

First, navigate to the `frontend` directory and install dependencies required for the Docker build.

```bash
# From the project root directory
cd frontend
npm install
cd ..
````

### 2\. Configure Environment Variables

Create a file named `.env` inside the `backend/` directory with the following content. These credentials are used by the Rust application to connect to the internal Docker services.

**`backend/.env`**

```
DATABASE_URL=postgres://user:password@db:5432/scheduler_db
REDIS_URL=redis://redis:6379/
LISTEN_ADDR=0.0.0.0:8000
```

### 3\. Build and Run the Stack

Run the following command from the project root directory (`distributed-job-scheduler/`):

```bash
docker compose up --build -d
```

This command will:

1.  Build the Rust backend image and the Next.js frontend image.
2.  Start Postgres, Redis, the Rust backend (API + Worker), and the Next.js frontend.
3.  The Rust backend will automatically run database migrations on startup.

### 4\. Access the Dashboard

Once all containers are running (this may take a minute), open your browser:

  * **Admin Dashboard:** `http://localhost:3000`

## 🔗 API Endpoints

The Rust backend exposes the following REST API endpoints:

| Method | Path | Description | Example Payload (`POST`) |
| :--- | :--- | :--- | :--- |
| `POST` | `/jobs` | Submits a new job to the queue. | `{"job_type": "process_data", "payload": {"user_id": 101}}` |
| `GET` | `/jobs` | Lists all jobs (up to 100). | N/A |
| `GET` | `/jobs/{id}` | Retrieves a single job by ID. | N/A |
| `GET` | `/ws` | WebSocket endpoint for real-time status updates. | N/A |

## 💡 Key Design Decisions

### **Concurrency and Reliability**

  * **Tokio Task Separation:** The REST API and the Background Worker run concurrently as separate tasks spawned by the Tokio runtime, ensuring API responsiveness is never blocked by long-running jobs.
  * **Atomic Job Pulling:** The worker uses Redis **`BLPOP`** on the `job_queue` list. This is the idiomatic distributed pattern to ensure only one worker retrieves a job ID, eliminating race conditions.
  * **Status Broadcasting:** Job status changes are pushed instantly via the **`tokio::sync::broadcast`** channel, notifying all connected WebSocket clients efficiently.

### **Code Structure (Backend)**

The Rust backend (`backend/src/`) follows a clean, decoupled structure:

  * **`main.rs`:** Sets up shared state (`AppState` via `Arc`), initializes pools, and spawns the main tasks (Server + Worker).
  * **`api/`:** Contains Axum handlers for REST endpoints.
  * **`job_worker/`:** Contains the main worker loop and the **`JobExecutor`** trait for decoupled job processing logic.
  * **`db/`:** Handles SQLx connections and queries (e.g., `create_new_job`, `get_job_by_id`).
  * **`ws/`:** Manages WebSocket connection upgrades and message forwarding from the broadcast channel.

<!-- end list -->
