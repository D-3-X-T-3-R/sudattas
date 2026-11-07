import { gqlAdmin } from "./graphql-client";

export interface UserRoleRow {
  roleId: string;
  roleName: string;
}

export async function fetchUserRoles(): Promise<UserRoleRow[]> {
  const data = await gqlAdmin<{ searchUserRole?: UserRoleRow[] }>(
    `mutation UserRoles { searchUserRole(input: { roleId: "0" }) { roleId roleName } }`
  );
  return data?.searchUserRole ?? [];
}

export async function createUserRole(roleName: string): Promise<UserRoleRow | null> {
  const data = await gqlAdmin<{ createUserRole?: UserRoleRow[] }>(
    `mutation CreateUserRole($input: NewUserRole!) { createUserRole(input: $input) { roleId roleName } }`,
    { input: { roleName: roleName.trim() } }
  );
  return data?.createUserRole?.[0] ?? null;
}

export async function updateUserRole(roleId: string, roleName: string): Promise<UserRoleRow | null> {
  const data = await gqlAdmin<{ updateUserRole?: UserRoleRow[] }>(
    `mutation UpdateUserRole($input: UserRoleMutation!) { updateUserRole(input: $input) { roleId roleName } }`,
    { input: { roleId, roleName: roleName.trim() } }
  );
  return data?.updateUserRole?.[0] ?? null;
}

export async function deleteUserRole(roleId: string): Promise<void> {
  await gqlAdmin(
    `mutation DeleteUserRole($input: DeleteUserRoleInput!) { deleteUserRole(input: $input) { roleId } }`,
    { input: { roleId } }
  );
}
