import {
  Attribute,
  BelongsTo,
  Default,
  HasMany,
  NotNull,
  PrimaryKey,
  Table,
} from "@sequelize/core/decorators-legacy";
import { ModelPO, PO } from "./po";
import {
  type CreationOptional,
  DataTypes,
  type HasManyGetAssociationsMixin,
  type NonAttribute,
} from "@sequelize/core";
import { JournalPO } from "./journal.po";

@Table({ modelName: "Entry" })
export class EntryPO extends ModelPO<EntryPO> {
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
  declare description: CreationOptional<string>;

  @Attribute(DataTypes.STRING)
  @NotNull
  declare type: EntryType;

  @Attribute(DataTypes.DATEONLY)
  @NotNull
  @Default(DataTypes.NOW)
  declare date: CreationOptional<Date>;

  @HasMany(() => EntryTagPO, "entryId")
  declare tags?: NonAttribute<EntryTagPO[]>;

  declare getTags: HasManyGetAssociationsMixin<EntryTagPO>;

  @HasMany(() => EntryTagPO, "entryId")
  declare entries?: NonAttribute<EntryItemPO[]>;

  declare getEntries: HasManyGetAssociationsMixin<EntryItemPO>;
}

@Table({ modelName: "EntryTag", timestamps: false })
export class EntryTagPO extends PO<EntryTagPO> {
  @Attribute(DataTypes.UUID)
  @NotNull
  @PrimaryKey
  declare entryId: string;

  @Attribute(DataTypes.STRING)
  @NotNull
  @PrimaryKey
  declare tag: string;
}

@Table({ modelName: "EntryItem", timestamps: false })
export class EntryItemPO extends PO<EntryItemPO> {
  @Attribute(DataTypes.UUID)
  @NotNull
  @PrimaryKey
  declare entryId: string;

  @Attribute(DataTypes.UUID)
  @NotNull
  @PrimaryKey
  declare accountId: string;

  @Attribute(DataTypes.FLOAT.UNSIGNED)
  @NotNull
  declare amount: number;

  @Attribute(DataTypes.FLOAT.UNSIGNED)
  declare price?: number;
}

export const ENTRY_TYPES = ["CHECK", "RECORD"] as const;

export type EntryType = (typeof ENTRY_TYPES)[number];
