# Stripe Skill — `uws stripe`

Gives AI agents read access (and carefully gated write access) to Stripe: balance, customers, invoices, charges, subscriptions, payment intents, products, prices, and events.

## Auth
```bash
export STRIPE_SECRET_KEY=sk_live_...   # or sk_test_... for testing
# or
export UWS_STRIPE_TOKEN=sk_live_...
```

**⚠ NEVER commit your Stripe secret key to source control.**
**⚠ Use test keys (`sk_test_...`) during development.**

## Quick Reference

| Resource | Method | Destructive | Description |
|---|---|---|---|
| `balance` | `get` | No | Current account balance |
| `balance` | `transactions` | No | Balance transaction history |
| `customers` | `list` | No | List customers |
| `customers` | `get` | No | Get customer by ID |
| `customers` | `create` | No | Create a customer |
| `customers` | `update` | No | Update a customer |
| `customers` | `delete` | **Yes** | Delete a customer |
| `customers` | `search` | No | Search customers |
| `invoices` | `list` | No | List invoices |
| `invoices` | `send` | No | Send invoice to customer |
| `invoices` | `void` | **Yes** | Void an invoice |
| `charges` | `list` | No | List charges |
| `charges` | `refund` | **Yes** | Refund a charge |
| `subscriptions` | `list` | No | List subscriptions |
| `subscriptions` | `cancel` | **Yes** | Cancel a subscription |
| `payment-intents` | `list` | No | List payment intents |
| `products` | `list` | No | List products |
| `prices` | `list` | No | List prices |
| `events` | `list` | No | List Stripe events |

## Examples

```bash
# Check account balance
uws stripe balance get

# List recent charges
uws stripe charges list --params '{"limit":"10"}'

# Search for a customer
uws stripe customers search --params '{"query":"email:alice@example.com"}'

# Get a customer's invoices
uws stripe invoices list --params '{"customer":"cus_123","status":"open"}'

# List active subscriptions
uws stripe subscriptions list --params '{"status":"active","limit":"20"}'

# Preview a refund (MANDATORY dry-run for destructive operations)
uws stripe charges refund --json '{"charge":"ch_123","amount":1000}' --dry-run

# Preview subscription cancellation
uws stripe subscriptions cancel --params '{"subscription_id":"sub_123"}' --dry-run

# List recent webhook events
uws stripe events list --params '{"limit":"10","type":"customer.subscription.updated"}'
```

## Agent Rules

1. **Always use `--dry-run` for destructive operations.** The SDK enforces this — a missing `--dry-run` on destructive calls returns an error.
2. **Always confirm with the user before issuing refunds or cancelling subscriptions.**
3. Use test keys (`sk_test_...`) for all non-production workflows.
4. Use `events list` to audit recent API activity before making changes.
5. Customer IDs start with `cus_`, invoice IDs with `in_`, charge IDs with `ch_`.
