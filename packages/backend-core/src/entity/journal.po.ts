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
  Default,
} from "@sequelize/core/decorators-legacy";
import { ModelPO, PO } from "./po";

@Table({ modelName: "Journal" })
export class JournalPO extends ModelPO<JournalPO> {
  @Attribute(DataTypes.STRING)
  @NotNull
  declare name: string;

  @Attribute(DataTypes.STRING)
  @NotNull
  @Default("")
  declare description: string;

  @Attribute(DataTypes.STRING)
  @NotNull
  declare unit: string;

  @Attribute(DataTypes.BOOLEAN)
  @NotNull
  @Default(false)
  declare archived: CreationOptional<boolean>;

  @HasMany(() => JournalTagPO, "journalId")
  declare tags?: NonAttribute<JournalTagPO[]>;

  declare getTags: HasManyGetAssociationsMixin<JournalTagPO>;
}

@Table({ modelName: "JournalTag", timestamps: false })
export class JournalTagPO extends PO<JournalTagPO> {
  @Attribute(DataTypes.UUID)
  @NotNull
  @PrimaryKey
  declare journalId: string;

  @Attribute(DataTypes.STRING)
  @NotNull
  @PrimaryKey
  declare tag: string;
}
