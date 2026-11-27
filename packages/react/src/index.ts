import { useQuery } from "@tanstack/react-query";
import type { UseQueryOptions, UseQueryResult } from "@tanstack/react-query";
import type { RwaAssetClient, ComplianceClient, GovernanceClient, KycRecord, Proposal } from "@stellarforge-protocol/sdk";

/**
 * Hook to fetch the balance of an owner address using RwaAssetClient.
 */
export function useBalance(
  client: RwaAssetClient | null | undefined,
  ownerAddress: string | null | undefined,
  options?: Omit<UseQueryOptions<bigint, Error, bigint, [string, string | null | undefined]>, "queryKey" | "queryFn">
): UseQueryResult<bigint, Error> {
  return useQuery({
    queryKey: ["rwa-balance", ownerAddress],
    queryFn: async () => {
      if (!client || !ownerAddress) {
        throw new Error("Client and ownerAddress are required");
      }
      return await client.balance(ownerAddress);
    },
    enabled: Boolean(client && ownerAddress),
    ...options,
  });
}

/**
 * Hook to fetch the KYC status of a subject address using ComplianceClient.
 */
export function useKycStatus(
  client: ComplianceClient | null | undefined,
  subjectAddress: string | null | undefined,
  options?: Omit<UseQueryOptions<KycRecord | null, Error, KycRecord | null, [string, string | null | undefined]>, "queryKey" | "queryFn">
): UseQueryResult<KycRecord | null, Error> {
  return useQuery({
    queryKey: ["kyc-status", subjectAddress],
    queryFn: async () => {
      if (!client || !subjectAddress) {
        throw new Error("Client and subjectAddress are required");
      }
      return await client.getKyc(subjectAddress);
    },
    enabled: Boolean(client && subjectAddress),
    ...options,
  });
}

/**
 * Hook to fetch a proposal by ID using GovernanceClient.
 */
export function useProposals(
  client: GovernanceClient | null | undefined,
  proposalId: bigint | number | null | undefined,
  options?: Omit<UseQueryOptions<Proposal | null, Error, Proposal | null, [string, string | null | undefined]>, "queryKey" | "queryFn">
): UseQueryResult<Proposal | null, Error> {
  const normalizedId = proposalId !== null && proposalId !== undefined ? BigInt(proposalId) : null;
  return useQuery({
    queryKey: ["governance-proposal", normalizedId?.toString()],
    queryFn: async () => {
      if (!client || normalizedId === null) {
        throw new Error("Client and proposalId are required");
      }
      return await client.getProposal(normalizedId);
    },
    enabled: Boolean(client && normalizedId !== null),
    ...options,
  });
}
