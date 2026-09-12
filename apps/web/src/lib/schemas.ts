import { z } from "zod";

export const credentialsSchema = z.object({
  email: z.email("Enter a valid email"),
  password: z
    .string()
    .min(8, "Password must be at least 8 characters")
    .max(128, "Password must be at most 128 characters"),
});

export type CredentialsValues = z.infer<typeof credentialsSchema>;

export const emailSchema = z.object({
  email: z.email("Enter a valid email"),
});

export type EmailValues = z.infer<typeof emailSchema>;

export const resetPasswordSchema = z.object({
  password: z
    .string()
    .min(8, "Password must be at least 8 characters")
    .max(128, "Password must be at most 128 characters"),
});

export type ResetPasswordValues = z.infer<typeof resetPasswordSchema>;

export const projectSchema = z.object({
  name: z
    .string()
    .min(1, "Name is required")
    .max(100, "Name must be at most 100 characters"),
  description: z
    .string()
    .max(2000, "Description must be at most 2000 characters"),
});

export type ProjectValues = z.infer<typeof projectSchema>;
