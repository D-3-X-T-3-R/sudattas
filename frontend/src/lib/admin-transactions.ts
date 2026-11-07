import { gqlAdmin } from "./graphql-client";

export interface TransactionRow {
  transactionId: string;
  userId: string;
  amountPaise: string;
  transactionDate: string;
  type: string;
}

const TRANSACTION_FIELDS = `transactionId userId amountPaise transactionDate type`;

export async function fetchTransactions(): Promise<TransactionRow[]> {
  const data = await gqlAdmin<{ searchTransaction?: TransactionRow[] }>(
    `mutation Transactions { searchTransaction(input: { transactionId: "0" }) { ${TRANSACTION_FIELDS} } }`
  );
  const rows = data?.searchTransaction ?? [];
  return [...rows].sort(
    (a, b) => new Date(b.transactionDate).getTime() - new Date(a.transactionDate).getTime()
  );
}

export async function createTransactionAdmin(params: {
  userId: string;
  amountPaise: string;
  type: string;
}): Promise<TransactionRow | null> {
  const data = await gqlAdmin<{ createTransaction?: TransactionRow[] }>(
    `mutation CreateTransaction($input: NewTransaction!) {
      createTransaction(input: $input) { ${TRANSACTION_FIELDS} }
    }`,
    { input: params }
  );
  return data?.createTransaction?.[0] ?? null;
}

export async function updateTransactionAdmin(params: {
  transactionId: string;
  userId?: string;
  amountPaise?: string;
  type?: string;
}): Promise<TransactionRow | null> {
  const data = await gqlAdmin<{ updateTransaction?: TransactionRow[] }>(
    `mutation UpdateTransaction($input: TransactionMutation!) {
      updateTransaction(input: $input) { ${TRANSACTION_FIELDS} }
    }`,
    { input: params }
  );
  return data?.updateTransaction?.[0] ?? null;
}

export async function deleteTransactionAdmin(transactionId: string): Promise<void> {
  await gqlAdmin(
    `mutation DeleteTransaction($input: DeleteTransactionInput!) {
      deleteTransaction(input: $input) { transactionId }
    }`,
    { input: { transactionId } }
  );
}
