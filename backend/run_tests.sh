#!/bin/bash

# Test Runner Script
# Conveniently run different types of tests for the backend

set -e

# Color output for better readability
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${GREEN}=== Backend Test Runner ===${NC}"

# Check if cargo is installed
if ! command -v cargo &> /dev/null; then
    echo -e "${RED}Error: cargo is not installed${NC}"
    exit 1
fi

# Parse command line arguments
TEST_TYPE="${1:-unit}"

case "$TEST_TYPE" in
    "unit")
        echo -e "${YELLOW}Running unit tests (Domain + Service layers)...${NC}"
        cargo test --test domain_tests --test service_tests
        ;;

    "integration")
        echo -e "${YELLOW}Running integration tests (Repository layer)...${NC}"
        echo -e "${YELLOW}Note: Requires DATABASE_URL to be set${NC}"

        if [ -z "$DATABASE_URL" ]; then
            echo -e "${RED}Error: DATABASE_URL environment variable is not set${NC}"
            echo "Example: export DATABASE_URL='postgres://user:password@localhost/test_db'"
            exit 1
        fi

        cargo test --test repository_tests -- --ignored
        ;;

    "all")
        echo -e "${YELLOW}Running all tests...${NC}"

        # Run unit tests
        echo -e "${GREEN}1. Unit tests${NC}"
        cargo test --test domain_tests --test service_tests

        # Run integration tests (if database is available)
        if [ -n "$DATABASE_URL" ]; then
            echo -e "${GREEN}2. Integration tests${NC}"
            cargo test --test repository_tests -- --ignored
        else
            echo -e "${YELLOW}Skipping integration tests (DATABASE_URL not set)${NC}"
        fi

        # Run common module tests
        echo -e "${GREEN}3. Common module tests${NC}"
        cargo test --test common
        ;;

    "coverage")
        echo -e "${YELLOW}Generating test coverage report...${NC}"

        if ! command -v cargo-tarpaulin &> /dev/null; then
            echo -e "${RED}Error: cargo-tarpaulin is not installed${NC}"
            echo "Install it with: cargo install cargo-tarpaulin"
            exit 1
        fi

        if [ -n "$DATABASE_URL" ]; then
            echo "Running with integration tests..."
            cargo tarpaulin --out Html --out Lcov -- --ignored
        else
            echo "Running without integration tests..."
            cargo tarpaulin --exclude-files tests/repository_tests.rs --out Html --out Lcov
        fi

        echo -e "${GREEN}Coverage report generated: tarpaulin-report.html${NC}"
        ;;

    "watch")
        echo -e "${YELLOW}Running tests in watch mode...${NC}"

        if ! command -v cargo-watch &> /dev/null; then
            echo -e "${RED}Error: cargo-watch is not installed${NC}"
            echo "Install it with: cargo install cargo-watch"
            exit 1
        fi

        cargo watch -x "test --test domain_tests --test service_tests"
        ;;

    "clean")
        echo -e "${YELLOW}Cleaning test artifacts...${NC}"
        cargo clean
        rm -f tarpaulin-report.html
        rm -f lcov.info
        echo -e "${GREEN}Done!${NC}"
        ;;

    "help"|"-h"|"--help")
        echo "Usage: $0 [TEST_TYPE]"
        echo ""
        echo "Test types:"
        echo "  unit         - Run unit tests (default)"
        echo "  integration  - Run integration tests (requires DATABASE_URL)"
        echo "  all          - Run all tests"
        echo "  coverage     - Generate coverage report (requires cargo-tarpaulin)"
        echo "  watch        - Run tests in watch mode (requires cargo-watch)"
        echo "  clean        - Clean test artifacts"
        echo "  help         - Show this help message"
        echo ""
        echo "Examples:"
        echo "  $0 unit"
        echo "  DATABASE_URL='postgres://...' $0 integration"
        echo "  $0 all"
        echo "  $0 coverage"
        ;;

    *)
        echo -e "${RED}Error: Unknown test type '$TEST_TYPE'${NC}"
        echo "Run '$0 help' for usage information"
        exit 1
        ;;
esac

echo -e "${GREEN}✓ Tests completed successfully!${NC}"
