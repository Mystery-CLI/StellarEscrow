import { ApiClient } from "../client";
import { startKyc } from "./kyc";
import { runAml } from "./aml";

export class ComplianceApi {
  constructor(private client: ApiClient) {}

  async startKyc(userId: string) {
    // later: call backend API
    return startKyc(userId);
  }

  async amlCheck(name: string) {
    const result = await runAml(name);

    return {
      status: result.flagged ? "UNDER_REVIEW" : "CLEAR",
      ...result,
    };
  }
}