import * as contracts from "../../contracts";

export const createEscrow = async (req, res) => {
  try {
    const result = await contracts.createEscrow(req.body);
    res.json(result);
  } catch (err) {
    res.status(500).json({ error: err.message });
  }
};