import { Attribute, Default, PrimaryKey, Table, Version } from "@sequelize/core/decorators-legacy";
import {
  type CreationOptional,
  DataTypes,
  type InferAttributes,
  type InferCreationAttributes,
  Model,
} from "@sequelize/core";

@Table.Abstract({ timestamps: false })
export abstract class PO<M extends PO<M>> extends Model<
  InferAttributes<M>,
  InferCreationAttributes<M>
> {}

@Table.Abstract({ timestamps: true })
export abstract class ModelPO<M extends ModelPO<M>> extends PO<M> {
  @PrimaryKey
  @Attribute(DataTypes.UUID)
  @Default(DataTypes.UUIDV4)
  declare id: CreationOptional<string>;

  declare createdAt: CreationOptional<Date>;

  declare updatedAt: CreationOptional<Date>;

  @Version
  declare version: CreationOptional<number>;
}
