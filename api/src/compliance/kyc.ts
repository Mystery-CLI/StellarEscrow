export async function startKyc(userId: string) {
  return {
    provider: "mock",
    sessionId: `kyc_${userId}`,
    status: "PENDING"
  };
}