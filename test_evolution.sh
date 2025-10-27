#!/bin/bash

# Simple test to verify genome sizes remain stable
# This runs a simulation for 1000 updates and checks that genomes don't shrink

cargo run --bin debug_test -- --updates 1000 2>&1 | grep -E "(Average genome|Population|Merit)" | tail -20

echo ""
echo "Test complete. Check that:"
echo "1. Average genome size is close to 50 (the ancestor size)"
echo "2. Population is growing (births > deaths)"
echo "3. If tasks evolve, merit should increase"
