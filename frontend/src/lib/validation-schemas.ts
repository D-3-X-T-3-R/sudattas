import { z } from "zod";
import {
  MAX_MONEY_PAISE,
  RUPEES_INPUT_REGEX,
  rupeesInputToPaise,
} from "@/lib/money";

export const PINCODE_REGEX = /^\d{6}$/;
export const PHONE_ALLOWED_REGEX = /^[0-9+\-()\s]+$/;

export const phoneSchema = z
  .string()
  .trim()
  .refine(
    (v) => v.length === 0 || (PHONE_ALLOWED_REGEX.test(v) && v.replace(/\D/g, "").length >= 10),
    "Phone must contain at least 10 digits and only valid phone characters"
  );

export const addressInputSchema = z.object({
  country: z.string().trim().min(2, "Country is required"),
  stateRegion: z.string().trim().min(2, "State/region is required"),
  city: z.string().trim().min(2, "City is required"),
  postalCode: z
    .string()
    .trim()
    .regex(PINCODE_REGEX, "Pincode must be exactly 6 digits"),
  road: z
    .string()
    .trim()
    .min(3, "Road/street is required")
    .max(500, "Road/street must be at most 500 characters"),
  apartmentNoOrName: z.string().trim().nullable().optional(),
});

// Accept positive rupee amount with up to 2 decimal places.
export const rupeesInputSchema = z
  .string()
  .trim()
  .regex(RUPEES_INPUT_REGEX, "Enter a valid amount (up to 2 decimals)")
  .refine((v) => rupeesInputToPaise(v) > 0, "Amount must be greater than 0")
  .refine(
    (v) => rupeesInputToPaise(v) <= MAX_MONEY_PAISE,
    "Amount exceeds supported maximum"
  );
