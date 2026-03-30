export async function runAml(name: string) {
  const riskScore = Math.floor(Math.random() * 100);

  return {
    riskScore,
    flagged: riskScore > 70
  };
}