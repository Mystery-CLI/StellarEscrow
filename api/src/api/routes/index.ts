import { Router } from "express";
import authRoutes from "./auth.routes";
import escrowRoutes from "./escrow.routes";

const router = Router();

router.use("/auth", authRoutes);
router.use("/escrow", escrowRoutes);

router.get("/health", (req, res) => {
  res.json({ status: "OK" });
});

export default router;