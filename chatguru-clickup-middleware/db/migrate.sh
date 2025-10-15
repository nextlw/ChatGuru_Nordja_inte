#!/bin/bash

# ============================================================================
# ChatGuru-ClickUp Middleware - GCP Cloud SQL Migration Script
# ============================================================================
# Version: 1.0
# Description: Apply schema and seed data to Cloud SQL PostgreSQL
# Usage: ./migrate.sh [local|gcp] [apply|rollback|status]
# ============================================================================

set -e  # Exit on error

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
SCHEMA_FILE="${SCRIPT_DIR}/schema.sql"
SEED_FILE="${SCRIPT_DIR}/seed_data.sql"

# GCP Configuration (from .env or environment)
GCP_PROJECT_ID="${GCP_PROJECT_ID:-buzzlightear}"
GCP_REGION="${GCP_REGION:-southamerica-east1}"
DB_INSTANCE_NAME="${DB_INSTANCE_NAME:-chatguru-middleware-db}"
DB_NAME="${DB_NAME:-chatguru_middleware}"
DB_USER="${DB_USER:-postgres}"

# Local Configuration
LOCAL_DB_HOST="${LOCAL_DB_HOST:-localhost}"
LOCAL_DB_PORT="${LOCAL_DB_PORT:-5432}"
LOCAL_DB_NAME="${LOCAL_DB_NAME:-chatguru_middleware}"
LOCAL_DB_USER="${LOCAL_DB_USER:-postgres}"

# ============================================================================
# Helper Functions
# ============================================================================

print_header() {
    echo -e "${BLUE}============================================================================${NC}"
    echo -e "${BLUE}$1${NC}"
    echo -e "${BLUE}============================================================================${NC}"
}

print_success() {
    echo -e "${GREEN}✓ $1${NC}"
}

print_error() {
    echo -e "${RED}✗ $1${NC}"
}

print_warning() {
    echo -e "${YELLOW}⚠ $1${NC}"
}

print_info() {
    echo -e "${BLUE}ℹ $1${NC}"
}

check_requirements() {
    print_info "Checking requirements..."

    if ! command -v psql &> /dev/null; then
        print_error "psql is not installed. Please install PostgreSQL client."
        exit 1
    fi

    if [[ "$ENVIRONMENT" == "gcp" ]] && ! command -v gcloud &> /dev/null; then
        print_error "gcloud CLI is not installed. Please install Google Cloud SDK."
        exit 1
    fi

    if [[ ! -f "$SCHEMA_FILE" ]]; then
        print_error "Schema file not found: $SCHEMA_FILE"
        exit 1
    fi

    if [[ ! -f "$SEED_FILE" ]]; then
        print_error "Seed data file not found: $SEED_FILE"
        exit 1
    fi

    print_success "All requirements met"
}

# ============================================================================
# Local Database Functions
# ============================================================================

migrate_local() {
    print_header "Applying Migration to LOCAL PostgreSQL"

    print_info "Database: $LOCAL_DB_NAME"
    print_info "Host: $LOCAL_DB_HOST:$LOCAL_DB_PORT"
    print_info "User: $LOCAL_DB_USER"

    # Test connection
    print_info "Testing database connection..."
    if ! PGPASSWORD="$LOCAL_DB_PASSWORD" psql -h "$LOCAL_DB_HOST" -p "$LOCAL_DB_PORT" -U "$LOCAL_DB_USER" -d "$LOCAL_DB_NAME" -c "SELECT 1;" &> /dev/null; then
        print_error "Cannot connect to local database. Is it running?"
        print_info "Try: cd db && docker-compose up -d"
        exit 1
    fi
    print_success "Connection successful"

    # Apply schema
    print_info "Applying schema..."
    PGPASSWORD="$LOCAL_DB_PASSWORD" psql -h "$LOCAL_DB_HOST" -p "$LOCAL_DB_PORT" -U "$LOCAL_DB_USER" -d "$LOCAL_DB_NAME" -f "$SCHEMA_FILE"
    print_success "Schema applied"

    # Apply seed data
    print_info "Applying seed data..."
    PGPASSWORD="$LOCAL_DB_PASSWORD" psql -h "$LOCAL_DB_HOST" -p "$LOCAL_DB_PORT" -U "$LOCAL_DB_USER" -d "$LOCAL_DB_NAME" -f "$SEED_FILE"
    print_success "Seed data applied"

    print_success "Local migration completed successfully!"
}

status_local() {
    print_header "LOCAL Database Status"

    print_info "Database: $LOCAL_DB_NAME"
    print_info "Host: $LOCAL_DB_HOST:$LOCAL_DB_PORT"
    echo ""

    PGPASSWORD="$LOCAL_DB_PASSWORD" psql -h "$LOCAL_DB_HOST" -p "$LOCAL_DB_PORT" -U "$LOCAL_DB_USER" -d "$LOCAL_DB_NAME" << 'EOF'
-- Database size
SELECT
    pg_database.datname AS database_name,
    pg_size_pretty(pg_database_size(pg_database.datname)) AS size
FROM pg_database
WHERE datname = current_database();

-- Table counts
SELECT 'teams' AS table_name, COUNT(*) AS rows FROM teams
UNION ALL SELECT 'spaces', COUNT(*) FROM spaces
UNION ALL SELECT 'folders', COUNT(*) FROM folders
UNION ALL SELECT 'lists', COUNT(*) FROM lists
UNION ALL SELECT 'categories', COUNT(*) FROM categories
UNION ALL SELECT 'subcategories', COUNT(*) FROM subcategories
UNION ALL SELECT 'category_subcategory_mapping', COUNT(*) FROM category_subcategory_mapping
UNION ALL SELECT 'activity_types', COUNT(*) FROM activity_types
UNION ALL SELECT 'status_options', COUNT(*) FROM status_options
UNION ALL SELECT 'client_requesters', COUNT(*) FROM client_requesters
UNION ALL SELECT 'attendant_aliases', COUNT(*) FROM attendant_aliases
UNION ALL SELECT 'folder_mapping', COUNT(*) FROM folder_mapping
UNION ALL SELECT 'list_cache', COUNT(*) FROM list_cache
ORDER BY table_name;

-- System config
SELECT * FROM system_config ORDER BY key;
EOF
}

# ============================================================================
# GCP Cloud SQL Functions
# ============================================================================

migrate_gcp() {
    print_header "Applying Migration to GCP Cloud SQL"

    print_info "Project: $GCP_PROJECT_ID"
    print_info "Region: $GCP_REGION"
    print_info "Instance: $DB_INSTANCE_NAME"
    print_info "Database: $DB_NAME"
    print_info "User: $DB_USER"

    # Check if gcloud is configured
    print_info "Checking gcloud configuration..."
    CURRENT_PROJECT=$(gcloud config get-value project 2>/dev/null)
    if [[ "$CURRENT_PROJECT" != "$GCP_PROJECT_ID" ]]; then
        print_warning "Current project is $CURRENT_PROJECT, switching to $GCP_PROJECT_ID"
        gcloud config set project "$GCP_PROJECT_ID"
    fi
    print_success "gcloud configured"

    # Test connection via Cloud SQL Proxy or direct connection
    print_info "Connecting to Cloud SQL..."

    # Create temporary files for SQL execution
    TMP_SCHEMA="/tmp/chatguru_schema_$$.sql"
    TMP_SEED="/tmp/chatguru_seed_$$.sql"
    cp "$SCHEMA_FILE" "$TMP_SCHEMA"
    cp "$SEED_FILE" "$TMP_SEED"

    # Apply schema via gcloud sql execute
    print_info "Applying schema via gcloud..."
    if gcloud sql connect "$DB_INSTANCE_NAME" --user="$DB_USER" --database="$DB_NAME" < "$TMP_SCHEMA"; then
        print_success "Schema applied successfully"
    else
        print_error "Failed to apply schema"
        rm -f "$TMP_SCHEMA" "$TMP_SEED"
        exit 1
    fi

    # Apply seed data
    print_info "Applying seed data via gcloud..."
    if gcloud sql connect "$DB_INSTANCE_NAME" --user="$DB_USER" --database="$DB_NAME" < "$TMP_SEED"; then
        print_success "Seed data applied successfully"
    else
        print_error "Failed to apply seed data"
        rm -f "$TMP_SCHEMA" "$TMP_SEED"
        exit 1
    fi

    # Cleanup
    rm -f "$TMP_SCHEMA" "$TMP_SEED"

    print_success "GCP Cloud SQL migration completed successfully!"
}

status_gcp() {
    print_header "GCP Cloud SQL Status"

    print_info "Project: $GCP_PROJECT_ID"
    print_info "Instance: $DB_INSTANCE_NAME"
    echo ""

    print_info "Instance details:"
    gcloud sql instances describe "$DB_INSTANCE_NAME" --format="table(
        name,
        region,
        databaseVersion,
        state,
        ipAddresses[0].ipAddress
    )"

    print_info "Databases:"
    gcloud sql databases list --instance="$DB_INSTANCE_NAME" --format="table(name,charset,collation)"

    print_info "Connecting to check table status..."
    gcloud sql connect "$DB_INSTANCE_NAME" --user="$DB_USER" --database="$DB_NAME" << 'EOF'
-- Table counts
SELECT 'teams' AS table_name, COUNT(*) AS rows FROM teams
UNION ALL SELECT 'spaces', COUNT(*) FROM spaces
UNION ALL SELECT 'folders', COUNT(*) FROM folders
UNION ALL SELECT 'lists', COUNT(*) FROM lists
UNION ALL SELECT 'categories', COUNT(*) FROM categories
UNION ALL SELECT 'subcategories', COUNT(*) FROM subcategories
UNION ALL SELECT 'category_subcategory_mapping', COUNT(*) FROM category_subcategory_mapping
UNION ALL SELECT 'activity_types', COUNT(*) FROM activity_types
UNION ALL SELECT 'status_options', COUNT(*) FROM status_options
UNION ALL SELECT 'client_requesters', COUNT(*) FROM client_requesters
UNION ALL SELECT 'attendant_aliases', COUNT(*) FROM attendant_aliases
UNION ALL SELECT 'folder_mapping', COUNT(*) FROM folder_mapping
UNION ALL SELECT 'list_cache', COUNT(*) FROM list_cache
ORDER BY table_name;
EOF
}

rollback_gcp() {
    print_header "Rollback GCP Cloud SQL (DROP ALL TABLES)"

    print_warning "This will DROP ALL TABLES in the database!"
    read -p "Are you sure you want to continue? (yes/no): " -r
    echo
    if [[ ! $REPLY =~ ^[Yy][Ee][Ss]$ ]]; then
        print_info "Rollback cancelled"
        exit 0
    fi

    print_info "Dropping all tables..."
    gcloud sql connect "$DB_INSTANCE_NAME" --user="$DB_USER" --database="$DB_NAME" << 'EOF'
DROP TABLE IF EXISTS task_cache CASCADE;
DROP TABLE IF EXISTS list_cache CASCADE;
DROP TABLE IF EXISTS folder_mapping CASCADE;
DROP TABLE IF EXISTS attendant_aliases CASCADE;
DROP TABLE IF EXISTS client_requesters CASCADE;
DROP TABLE IF EXISTS status_options CASCADE;
DROP TABLE IF EXISTS activity_types CASCADE;
DROP TABLE IF EXISTS category_subcategory_mapping CASCADE;
DROP TABLE IF EXISTS subcategories CASCADE;
DROP TABLE IF EXISTS categories CASCADE;
DROP TABLE IF EXISTS custom_field_types CASCADE;
DROP TABLE IF EXISTS lists CASCADE;
DROP TABLE IF EXISTS folders CASCADE;
DROP TABLE IF EXISTS spaces CASCADE;
DROP TABLE IF EXISTS teams CASCADE;
DROP TABLE IF EXISTS system_config CASCADE;
DROP EXTENSION IF EXISTS "uuid-ossp";
EOF

    print_success "All tables dropped successfully"
}

# ============================================================================
# Main Script
# ============================================================================

show_usage() {
    echo "Usage: $0 [local|gcp] [apply|rollback|status]"
    echo ""
    echo "Environments:"
    echo "  local    - Apply to local PostgreSQL (docker-compose)"
    echo "  gcp      - Apply to GCP Cloud SQL"
    echo ""
    echo "Commands:"
    echo "  apply    - Apply schema and seed data"
    echo "  rollback - Drop all tables (GCP only)"
    echo "  status   - Show database status"
    echo ""
    echo "Examples:"
    echo "  $0 local apply    # Apply to local database"
    echo "  $0 gcp apply      # Apply to GCP Cloud SQL"
    echo "  $0 local status   # Check local database status"
    echo "  $0 gcp status     # Check GCP database status"
    echo "  $0 gcp rollback   # Drop all tables in GCP"
    exit 1
}

# Parse arguments
ENVIRONMENT="${1:-}"
COMMAND="${2:-apply}"

if [[ -z "$ENVIRONMENT" ]] || [[ ! "$ENVIRONMENT" =~ ^(local|gcp)$ ]]; then
    show_usage
fi

if [[ ! "$COMMAND" =~ ^(apply|rollback|status)$ ]]; then
    show_usage
fi

# Check requirements
check_requirements

# Execute command
case "$ENVIRONMENT-$COMMAND" in
    local-apply)
        migrate_local
        ;;
    local-status)
        status_local
        ;;
    local-rollback)
        print_error "Rollback not supported for local. Use: docker-compose down -v"
        exit 1
        ;;
    gcp-apply)
        migrate_gcp
        ;;
    gcp-status)
        status_gcp
        ;;
    gcp-rollback)
        rollback_gcp
        ;;
    *)
        show_usage
        ;;
esac

print_success "Operation completed successfully!"
