import { describe, it, expect } from "vitest";
import {
  toStroops,
  fromStroops,
  sha256Hex,
  hexToBytes32,
  isValidStellarAddress,
  isValidContractId,
  truncateAddress,
} from "../src/utils.js";

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
});
