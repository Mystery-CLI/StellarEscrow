import jwt from "jsonwebtoken";

export const login = async (req, res) => {
  const { email } = req.body;

  // TODO: connect with real user system later
  const token = jwt.sign({ email }, process.env.JWT_SECRET!, {
    expiresIn: "1h",
  });

  res.json({ token });
};