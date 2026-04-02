export type ApiEnvelope<T> = {
  ok: boolean;
  data: T | null;
  errorCode: string | null;
  message: string | null;
  fieldErrors: Record<string, string> | null;
  retryable: boolean;
};

export class ApiEnvelopeError extends Error {
  readonly status: number;
  readonly errorCode: string | null;
  readonly fieldErrors: Record<string, string> | null;
  readonly retryable: boolean;

  constructor(params: {
    message: string;
    status: number;
    errorCode: string | null;
    fieldErrors: Record<string, string> | null;
    retryable: boolean;
  }) {
    super(params.message);
    this.name = "ApiEnvelopeError";
    this.status = params.status;
    this.errorCode = params.errorCode;
    this.fieldErrors = params.fieldErrors;
    this.retryable = params.retryable;
  }
}

export async function fetchApiEnvelope<T>(
  input: RequestInfo | URL,
  init?: RequestInit
): Promise<T> {
  const response = await fetch(input, init);
  const text = await response.text();

  let parsed: ApiEnvelope<T> | null = null;
  try {
    parsed = JSON.parse(text) as ApiEnvelope<T>;
  } catch {
    throw new ApiEnvelopeError({
      message: text || `HTTP ${response.status}`,
      status: response.status,
      errorCode: null,
      fieldErrors: null,
      retryable: response.status >= 500,
    });
  }

  if (!parsed || typeof parsed.ok !== "boolean") {
    throw new ApiEnvelopeError({
      message: `Invalid API response (HTTP ${response.status})`,
      status: response.status,
      errorCode: null,
      fieldErrors: null,
      retryable: response.status >= 500,
    });
  }

  if (!response.ok || !parsed.ok) {
    throw new ApiEnvelopeError({
      message: parsed.message || `HTTP ${response.status}`,
      status: response.status,
      errorCode: parsed.errorCode,
      fieldErrors: parsed.fieldErrors,
      retryable: parsed.retryable,
    });
  }

  return parsed.data as T;
}

