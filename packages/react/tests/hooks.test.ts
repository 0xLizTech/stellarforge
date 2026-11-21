import { describe, it, expect, vi } from "vitest";
import { useBalance, useKycStatus, useProposals } from "../src/index.js";

describe("@stellarforge-protocol/react hooks", () => {
  it("exports useBalance, useKycStatus, and useProposals", () => {
    expect(typeof useBalance).toBe("function");
    expect(typeof useKycStatus).toBe("function");
    expect(typeof useProposals).toBe("function");
  });
});
