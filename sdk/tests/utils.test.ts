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

  it("truncates extra decimal places", () => {
    expect(toStroops("1.12345678")).toBe(11_234_567n);
  });

  it("handles string integers", () => {
    expect(toStroops("42")).toBe(420_000_000n);
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
