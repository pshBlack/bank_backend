#!/bin/bash

BASE_URL="http://127.0.0.1:3000"

echo "🧪 Testing Bank API..."
echo ""

# 1. Registration Alice
echo "1️⃣ Creating user Alice..."
ALICE_RESPONSE=$(curl -s -X POST $BASE_URL/register \
  -H "Content-Type: application/json" \
  -d '{"username": "alice", "password": "alice123"}')
echo "Response: $ALICE_RESPONSE"
ALICE_ID=$(echo $ALICE_RESPONSE | jq -r '.id')
echo "Alice ID: $ALICE_ID"
echo ""

# 2. Registration Bob
echo "2️⃣ Creating user Bob..."
BOB_RESPONSE=$(curl -s -X POST $BASE_URL/register \
  -H "Content-Type: application/json" \
  -d '{"username": "bob", "password": "bob123"}')
echo "Response: $BOB_RESPONSE"
BOB_ID=$(echo $BOB_RESPONSE | jq -r '.id')
echo "Bob ID: $BOB_ID"
echo ""

# 3. Login Alice
echo "3️⃣ Login Alice..."
curl -s -X POST $BASE_URL/login \
  -H "Content-Type: application/json" \
  -d '{"username": "alice", "password": "alice123"}' | jq
echo ""

# 4. Create acc for Alice
echo "4️⃣ Creating account for Alice..."
ALICE_ACCOUNT_RESPONSE=$(curl -s -X POST $BASE_URL/accounts \
  -H "Content-Type: application/json" \
  -d "{\"user_id\": \"$ALICE_ID\"}")
echo "Response: $ALICE_ACCOUNT_RESPONSE"
ALICE_ACCOUNT=$(echo $ALICE_ACCOUNT_RESPONSE | jq -r '.id')
echo "Alice Account: $ALICE_ACCOUNT"
echo ""

# 5. Create acc for Bob
echo "5️⃣ Creating account for Bob..."
BOB_ACCOUNT_RESPONSE=$(curl -s -X POST $BASE_URL/accounts \
  -H "Content-Type: application/json" \
  -d "{\"user_id\": \"$BOB_ID\"}")
echo "Response: $BOB_ACCOUNT_RESPONSE"
BOB_ACCOUNT=$(echo $BOB_ACCOUNT_RESPONSE | jq -r '.id')
echo "Bob Account: $BOB_ACCOUNT"
echo ""

# 6. Adding money to Alice
echo "6️⃣ Adding 1000 to Alice account..."
curl -s -X POST $BASE_URL/addmoney \
  -H "Content-Type: application/json" \
  -d "{\"account_id\": \"$ALICE_ACCOUNT\", \"amount\": \"1000.00\"}" | jq
echo ""

# 7. Adding money to Bob
echo "7️⃣ Adding 500 to Bob account..."
curl -s -X POST $BASE_URL/addmoney \
  -H "Content-Type: application/json" \
  -d "{\"account_id\": \"$BOB_ACCOUNT\", \"amount\": \"500.00\"}" | jq
echo ""

# 8. Transfering money
echo "8️⃣ Transfer 250.50 from Alice to Bob..."
curl -s -X POST $BASE_URL/transactions \
  -H "Content-Type: application/json" \
  -d "{\"from_account\": \"$ALICE_ACCOUNT\", \"to_account\": \"$BOB_ACCOUNT\", \"amount\": \"250.50\"}" | jq
echo ""

# 9. Show balances after transf. test
echo "Alice accounts (should be 749.50):"
curl -s $BASE_URL/users/$ALICE_ID/accounts | jq
echo ""

echo "Bob accounts (should be 750.50):"
curl -s $BASE_URL/users/$BOB_ID/accounts | jq
echo ""

# 10. History of Transactions
echo "Alice transaction history:"
curl -s $BASE_URL/accounts/$ALICE_ACCOUNT/transactions | jq
echo ""

echo "Tests completed!"