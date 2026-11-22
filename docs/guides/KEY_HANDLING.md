# SDK Security Guide: Key Handling and Signing (#212)

## Best Practices
1. **Never Hardcode Secrets**: Store secret keys in environment variables or hardware security modules (HSM).
2. **Use Ephemeral Signers**: Utilize session keys and scoped allowances rather than root admin keypairs.
3. **Validate Addresses Before Signing**: Use \`validateAddress\` to prevent accidental signature leaks to invalid destinations.
4. **Sponsored Transactions**: Verify fee-payer authorization trees before broadcasting sponsored envelopes.
