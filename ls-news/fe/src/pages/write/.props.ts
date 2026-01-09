// Auto-generated from rs/src/pages/write.rs

import { z } from "zod";

export const PropsSchema = z.discriminatedUnion("t", [
    z.object({
    t: z.literal("Ok"),
  })
  ]);

export type Props = z.infer<typeof PropsSchema>;
