import {
  type CreationOptional,
  DataTypes,
  type HasManyGetAssociationsMixin,
  type NonAttribute,
} from "@sequelize/core";
import {
  Attribute,
  NotNull,
  PrimaryKey,
  Table,
  HasMany,
  BelongsTo,
  Default,
} from "@sequelize/core/decorators-legacy";
import { ModelPO, PO } from "./po";
import { JournalPO } from "./journal.po";

@Table({ modelName: "Account" })
export class AccountPO extends ModelPO<AccountPO> {
  @BelongsTo(() => JournalPO, "journalId")
  declare journal?: NonAttribute<JournalPO>;

  @Attribute(DataTypes.UUID)
  @NotNull
  declare journalId: string;

  @Attribute(DataTypes.STRING)
  @NotNull
  declare name: string;

  @Attribute(DataTypes.STRING)
  @NotNull
  @Default("")
  declare description: string;

  @Attribute(DataTypes.STRING)
  @NotNull
  declare type: AccountType;

  @Attribute(DataTypes.STRING)
  @NotNull
  declare unit: string;

  @Attribute(DataTypes.BOOLEAN)
  @NotNull
  @Default(false)
  declare archived: CreationOptional<boolean>;

  @HasMany(() => AccountTagPO, "accountId")
  declare tags?: NonAttribute<AccountTagPO[]>;

  declare getTags: HasManyGetAssociationsMixin<AccountTagPO>;
}

@Table({ modelName: "AccountTag", timestamps: false })
export class AccountTagPO extends PO<AccountTagPO> {
  @Attribute(DataTypes.UUID)
  @NotNull
  @PrimaryKey
  declare accountId: string;

  @Attribute(DataTypes.STRING)
  @NotNull
  @PrimaryKey
  declare tag: string;
}

export const ACCOUNT_TYPES = ["INCOME", "EXPENSE", "ASSET", "LIABILITY", "EQUITY"] as const;

export type AccountType = (typeof ACCOUNT_TYPES)[number];
