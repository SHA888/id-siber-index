#!/bin/bash
# Load test fixtures into the database
# Usage: bash tests/fixtures/load_fixtures.sh [setup|teardown|status]

set -e

# Get database URL from environment or use defaults
DB_HOST="${POSTGRES_HOST:-localhost}"
DB_PORT="${POSTGRES_PORT:-5432}"
DB_NAME="${POSTGRES_DB:-id_siber_index}"
DB_USER="${POSTGRES_USER:-postgres}"

# Allow password via env var (PGPASSWORD for psql)
export PGPASSWORD="${POSTGRES_PASSWORD:-}"

FIXTURE_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
SETUP_SQL="${FIXTURE_DIR}/test_incidents.sql"
TEARDOWN_SQL="${FIXTURE_DIR}/test_incidents_rollback.sql"

# Color output
GREEN='\033[0;32m'
RED='\033[0;31m'
NC='\033[0m' # No Color

psql_cmd() {
    psql -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" -d "$DB_NAME" "$@"
}

load_fixtures() {
    echo "Loading test fixtures into $DB_NAME..."
    if psql_cmd < "$SETUP_SQL"; then
        echo -e "${GREEN}✓ Test fixtures loaded successfully${NC}"
        return 0
    else
        echo -e "${RED}✗ Failed to load test fixtures${NC}"
        return 1
    fi
}

cleanup_fixtures() {
    echo "Cleaning up test fixtures from $DB_NAME..."
    if psql_cmd < "$TEARDOWN_SQL"; then
        echo -e "${GREEN}✓ Test fixtures cleaned up successfully${NC}"
        return 0
    else
        echo -e "${RED}✗ Failed to clean up test fixtures${NC}"
        return 1
    fi
}

check_fixtures() {
    echo "Checking test fixtures status..."
    count=$(psql_cmd -t -c "SELECT COUNT(*) FROM incidents WHERE id IN (
        '11111111-1111-1111-1111-111111111111'::uuid,
        '22222222-2222-2222-2222-222222222222'::uuid,
        '33333333-3333-3333-3333-333333333333'::uuid,
        '44444444-4444-4444-4444-444444444444'::uuid,
        '55555555-5555-5555-5555-555555555555'::uuid,
        '66666666-6666-6666-6666-666666666666'::uuid,
        '77777777-7777-7777-7777-777777777777'::uuid,
        '88888888-8888-8888-8888-888888888888'::uuid,
        '99999999-9999-9999-9999-999999999999'::uuid,
        'aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa'::uuid
    );" | xargs)

    if [ "$count" -eq 10 ]; then
        echo -e "${GREEN}✓ All 10 test fixtures are present${NC}"
        return 0
    else
        echo "Found $count test fixtures (expected 10)"
        return 1
    fi
}

case "${1:-setup}" in
    setup)
        load_fixtures
        ;;
    teardown|cleanup)
        cleanup_fixtures
        ;;
    status|check)
        check_fixtures
        ;;
    *)
        echo "Usage: $0 [setup|teardown|status]"
        echo ""
        echo "Commands:"
        echo "  setup   - Load test fixtures into database (default)"
        echo "  teardown - Remove test fixtures from database"
        echo "  status  - Check if test fixtures are present"
        exit 1
        ;;
esac
