import { describe, it, expect } from "vitest";
import { Account, Keypair, MuxedAccount, StrKey } from "@stellar/stellar-sdk";
import {
  toStroops,
  fromStroops,
  formatAmount,
  sha256Hex,
  hexToBytes32,
  isValidStellarAddress,
  isValidContractId,
  isValidMuxedAddress,
  parseMuxedAddress,
  validateAddress,
  truncateAddress,
} from "../src/utils.js";
import { InvalidAddressError } from "../src/errors.js";

describe("toStroops", () => {
  it("converts integer amounts", () => {
    expect(toStroops(1)).toBe(10_000_000n);
    expect(toStroops(100)).toBe(1_000_000_000n);
  });

  it("converts fractional amounts", () => {
    expect(toStroops("1.5")).toBe(15_000_000n);
    expect(toStroops("0.0000001")).toBe(1n);
    expect(toStroops("1.1234567")).toBe(11_234_567n);
  });

  // IR-06: truncating here silently moved less value than the caller typed.
  it("rejects precision beyond the asset's decimals", () => {
    expect(() => toStroops("1.12345678")).toThrow(/8 decimal places, more than the 7 allowed/);
  });

  it("accepts extra decimal places that are all zeros", () => {
    expect(toStroops("1.50000000")).toBe(15_000_000n);
  });

  it("converts negative amounts", () => {
    expect(toStroops("-0.5")).toBe(-5_000_000n);
    expect(toStroops(-3)).toBe(-30_000_000n);
  });

  it("handles string integers", () => {
    expect(toStroops("42")).toBe(420_000_000n);
  });
});

// IR-06. Each of these used to convert to some amount rather than fail.
describe("toStroops rejects input it would otherwise misread", () => {
  it.each(["1.2.3", "0x10", "1e3", " 1", "1 ", "1.", ".5", "", "abc", "1,000", "+1", "--1"])(
    "rejects the string %j",
    (input) => {
      expect(() => toStroops(input)).toThrow(/Invalid amount/);
    },
  );

  it.each([12345678901234567890, 1.5, 0.1 + 0.2, Number.NaN, Number.POSITIVE_INFINITY, 1e21])(
    "rejects the number %s, which is not a safe integer",
    (input) => {
      expect(() => toStroops(input)).toThrow(/not a safe integer/);
    },
  );

  it.each([-1, 19, 1.5])("rejects decimals of %s", (decimals) => {
    expect(() => toStroops("1", decimals)).toThrow(/decimals must be an integer/);
  });
});

describe("fromStroops", () => {
  it("converts back to decimal", () => {
    expect(fromStroops(10_000_000n)).toBe("1");
    expect(fromStroops(15_000_000n)).toBe("1.5");
    expect(fromStroops(1n)).toBe("0.0000001");
  });

  it("round-trips correctly", () => {
    const amounts = ["1", "42.5", "0.0000001", "1000000"];
    for (const a of amounts) {
      expect(fromStroops(toStroops(a))).toBe(a);
    }
  });
});

describe("toStroops with custom decimals", () => {
  it("scales by the given number of decimals", () => {
    expect(toStroops("1.5", 18)).toBe(1_500_000_000_000_000_000n);
    expect(toStroops("1", 2)).toBe(100n);
  });
});

describe("fromStroops with custom decimals", () => {
  it("round-trips at 18 decimals", () => {
    expect(fromStroops(toStroops("1.5", 18), 18)).toBe("1.5");
  });

  // IR-06: at zero decimals every digit is whole units. This used to render
  // 100 tokens as "0.1".
  it("renders whole units at zero decimals", () => {
    expect(fromStroops(100n, 0)).toBe("100");
    expect(fromStroops(0n, 0)).toBe("0");
    expect(fromStroops(toStroops("250", 0), 0)).toBe("250");
  });

  it("keeps integer trailing zeros", () => {
    expect(fromStroops(100_000_000n)).toBe("10");
  });

  it("rejects decimals outside the contract's range", () => {
    expect(() => fromStroops(1n, 19)).toThrow(/decimals must be an integer/);
  });
});

describe("fromStroops with negative amounts", () => {
  // IR-06: the sign used to be padded into the middle, as "0.00000-5".
  it("puts the sign in front", () => {
    expect(fromStroops(-5n)).toBe("-0.0000005");
    expect(fromStroops(-15_000_000n)).toBe("-1.5");
  });

  it("round-trips", () => {
    for (const a of ["-1", "-42.5", "-0.0000001"]) {
      expect(fromStroops(toStroops(a))).toBe(a);
    }
  });
});

describe("sha256Hex", () => {
  it("returns 64-character hex string", () => {
    const hash = sha256Hex("hello world");
    expect(hash).toHaveLength(64);
    expect(hash).toMatch(/^[0-9a-f]+$/);
  });

  it("is deterministic", () => {
    expect(sha256Hex("test")).toBe(sha256Hex("test"));
  });

  it("differs for different inputs", () => {
    expect(sha256Hex("a")).not.toBe(sha256Hex("b"));
  });
});

describe("hexToBytes32", () => {
  it("converts 64-char hex to 32 bytes", () => {
    const hex = "a".repeat(64);
    const bytes = hexToBytes32(hex);
    expect(bytes).toHaveLength(32);
    expect(bytes.every((b) => b === 0xaa)).toBe(true);
  });

  it("throws for wrong length", () => {
    expect(() => hexToBytes32("abc")).toThrow("Expected 64-char hex string");
  });

  // IR-06: non-hex digits used to become zero bytes without complaint.
  it("throws for non-hex digits of the right length", () => {
    expect(() => hexToBytes32("zz".repeat(32))).toThrow("Expected 64-char hex string");
    expect(() => hexToBytes32("0x" + "a".repeat(62))).toThrow("Expected 64-char hex string");
  });

  it("accepts uppercase hex", () => {
    expect(hexToBytes32("AB".repeat(32)).every((b) => b === 0xab)).toBe(true);
  });
});

describe("isValidStellarAddress", () => {
  it("rejects garbage strings", () => {
    expect(isValidStellarAddress("not-an-address")).toBe(false);
    expect(isValidStellarAddress("")).toBe(false);
  });

  it("rejects C-addresses", () => {
    // C-addresses are contract IDs, not public keys
    expect(isValidStellarAddress("C" + "A".repeat(55))).toBe(false);
  });
});

describe("isValidContractId", () => {
  it("rejects garbage strings", () => {
    expect(isValidContractId("not-a-contract")).toBe(false);
  });
});

describe("truncateAddress", () => {
  it("truncates long addresses", () => {
    const addr = "G" + "A".repeat(55);
    const truncated = truncateAddress(addr);
    expect(truncated).toContain("...");
    expect(truncated.length).toBeLessThan(addr.length);
  });

  it("does not truncate short strings", () => {
    expect(truncateAddress("GABC...DEFG")).toBe("GABC...DEFG");
  });

  describe("mutation edge cases", () => {
    it("handles zero and extreme stroop values deterministically", () => {
      expect(toStroops("0", 7)).toBe(0n);
      expect(fromStroops(0n, 7)).toBe("0");
      expect(toStroops("0.0000001", 7)).toBe(1n);
    });
  });
});

describe("decimals awareness across 0 to 18 decimals (#176)", () => {
  const fractionDigits = "123456789123456789";

  for (let d = 0; d <= 18; d++) {
    it(`round-trips amounts exactly at ${d} decimals`, () => {
      // Whole amounts
      expect(fromStroops(toStroops("100", d), d)).toBe("100");
      expect(fromStroops(toStroops("0", d), d)).toBe("0");
      expect(fromStroops(toStroops("1", d), d)).toBe("1");
      expect(toStroops(100n, d)).toBe(toStroops("100", d));

      if (d > 0) {
        // Exact fraction matching decimal depth
        const frac = fractionDigits.slice(0, d);
        const amount = `1.${frac}`;
        expect(fromStroops(toStroops(amount, d), d)).toBe(amount);

        // Reject excess precision
        const excess = `1.${"0".repeat(d)}1`;
        expect(() => toStroops(excess, d)).toThrow(
          new RegExp(`Amount ${excess} has ${d + 1} decimal places, more than the ${d} allowed`),
        );
      } else {
        // Zero decimals rejects any fraction with non-zeros
        expect(() => toStroops("1.1", 0)).toThrow(
          /Amount 1\.1 has 1 decimal places, more than the 0 allowed/,
        );
        // But accepts fractional zero
        expect(toStroops("1.000", 0)).toBe(1n);
      }
    });
  }

  it("handles bigint amounts in toStroops", () => {
    expect(toStroops(50n, 7)).toBe(500_000_000n);
    expect(toStroops(0n, 2)).toBe(0n);
    expect(toStroops(-10n, 3)).toBe(-10_000n);
  });
});

describe("formatAmount display helper (#177)", () => {
  it("formats exact 1e30 values without floating-point precision loss", () => {
    // 1e30 with 0 decimals
    const largeAmount0 = 10n ** 30n;
    expect(formatAmount(largeAmount0, 0, "en-US")).toBe(
      "1,000,000,000,000,000,000,000,000,000,000",
    );

    // 1e30 + 1 with 0 decimals (proves exactness beyond float53 max safe integer)
    const largePlusOne = 10n ** 30n + 1n;
    expect(formatAmount(largePlusOne, 0, "en-US")).toBe(
      "1,000,000,000,000,000,000,000,000,000,001",
    );

    // 1e30 tokens with 7 decimals (base units = 1e37)
    const largeAmount7 = 10n ** 37n;
    expect(formatAmount(largeAmount7, 7, "en-US")).toBe(
      "1,000,000,000,000,000,000,000,000,000,000",
    );

    // String input with 1e30 integer part and fractional part
    const str1e30 = "1000000000000000000000000000000.12345";
    expect(formatAmount(str1e30, 7, "en-US")).toBe(
      "1,000,000,000,000,000,000,000,000,000,000.12345",
    );
  });

  it("formats locale-aware digit groupings and decimal separators", () => {
    const amount = 1_234_567_890_123n; // 123,456.7890123 with 7 decimals
    expect(formatAmount(amount, 7, "en-US")).toBe("123,456.7890123");
    expect(formatAmount(amount, 7, "de-DE")).toBe("123.456,7890123");

    // String input
    expect(formatAmount("1234567.89", 7, "en-US")).toBe("1,234,567.89");
    expect(formatAmount("1234567.89", 7, "de-DE")).toBe("1.234.567,89");
  });

  it("formats negative amounts correctly across locales", () => {
    // Negative whole amount
    expect(formatAmount(-10_000_000n, 7, "en-US")).toBe("-1");
    // Negative fractional amount
    expect(formatAmount(-15_000_000n, 7, "en-US")).toBe("-1.5");
    expect(formatAmount(-15_000_000n, 7, "de-DE")).toBe("-1,5");
    // Negative fractional amount between 0 and -1
    expect(formatAmount(-500_000n, 7, "en-US")).toBe("-0.05");
    expect(formatAmount("-0.05", 7, "en-US")).toBe("-0.05");
  });

  it("formats zero amounts correctly", () => {
    expect(formatAmount(0n, 7)).toBe("0");
    expect(formatAmount("0", 7)).toBe("0");
    expect(formatAmount(0, 7)).toBe("0");
  });

  it("formats amounts across various decimal settings (0 to 18)", () => {
    expect(formatAmount(100n, 0)).toBe("100");
    expect(formatAmount(1_500_000_000_000_000_000n, 18)).toBe("1.5");
  });

  it("rejects unsafe numbers and invalid inputs", () => {
    expect(() => formatAmount(1.5, 7)).toThrow(/not a safe integer/);
    expect(() => formatAmount(1e25, 7)).toThrow(/not a safe integer/);
    expect(() => formatAmount("invalid", 7)).toThrow(/Invalid amount/);
    expect(() => formatAmount("1.12345678", 7)).toThrow(/more than the 7 allowed/);
    expect(() => formatAmount(100n, 19)).toThrow(/decimals must be an integer from 0 to 18/);
  });
});

describe("isValidMuxedAddress and parseMuxedAddress (#179)", () => {
  const kp = Keypair.random();
  const baseGAddr = kp.publicKey();
  const muxed = new MuxedAccount(new Account(baseGAddr, "0"), "42");
  const mAddr = muxed.accountId();

  it("correctly identifies valid M-addresses", () => {
    expect(isValidMuxedAddress(mAddr)).toBe(true);
    expect(isValidMuxedAddress(baseGAddr)).toBe(false);
    expect(isValidMuxedAddress("C" + "A".repeat(55))).toBe(false);
    expect(isValidMuxedAddress("garbage")).toBe(false);
    expect(isValidMuxedAddress("")).toBe(false);
  });

  it("parses valid M-addresses into base G-address and 64-bit ID", () => {
    const parsed = parseMuxedAddress(mAddr);
    expect(parsed).not.toBeNull();
    expect(parsed?.address).toBe(baseGAddr);
    expect(parsed?.id).toBe(42n);
  });

  it("parses M-addresses with 64-bit max int ID", () => {
    const maxMuxedId = (1n << 64n) - 1n;
    const maxMuxed = new MuxedAccount(
      new Account(baseGAddr, "0"),
      maxMuxedId.toString(),
    );
    const parsed = parseMuxedAddress(maxMuxed.accountId());
    expect(parsed).not.toBeNull();
    expect(parsed?.address).toBe(baseGAddr);
    expect(parsed?.id).toBe(maxMuxedId);
  });

  it("returns null when parsing non-muxed addresses", () => {
    expect(parseMuxedAddress(baseGAddr)).toBeNull();
    expect(parseMuxedAddress("C" + "A".repeat(55))).toBeNull();
    expect(parseMuxedAddress("invalid")).toBeNull();
  });
});

describe("validateAddress", () => {
  it("accepts valid G, C, and M addresses", () => {
    const keypair = Keypair.random();
    const muxed = new MuxedAccount(new Account(keypair.publicKey(), "0"), "42").accountId();
    validateAddress(keypair.publicKey());
    validateAddress(StrKey.encodeContract(new Uint8Array(32).fill(1)));
    validateAddress(muxed);
  });

  it("rejects invalid addresses with the optional parameter name", () => {
    expect(() => validateAddress("M" + "A".repeat(68), "recipient")).toThrow(InvalidAddressError);
    let caught: unknown;
    try {
      validateAddress("bad", "recipient");
    } catch (error) {
      caught = error;
    }
    expect(caught).toBeInstanceOf(InvalidAddressError);
    expect((caught as InvalidAddressError).parameterName).toBe("recipient");
  });
});
