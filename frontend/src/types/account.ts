export enum Gender {
  MALE = 0,
  FEMALE = 1,
}

export interface Account {
  id: number;
  nick_name?: string;
  email?: string;
  birth_date?: string;
  gender?: Gender;
}

export interface ResponseAccountAndToken {
  account: Account;
  token: string;
}
