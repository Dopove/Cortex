import express, { Request, Response } from 'express';

const app = express();
const PORT = process.env.PORT || 3000;

app.get('/', (req: Request, res: Response) => {
  res.json({ message: 'Hello from TypeScript + Express!' });
});

app.listen(PORT, () => {
  console.log(`TypeScript server running on port ${PORT}`);
});
