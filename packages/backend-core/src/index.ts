import "reflect-metadata";
import "dotenv";

import { createLogger, format, transports, Logger } from "winston";
import { container } from "tsyringe";
import { Sequelize, type Options as SequelizeOptions } from "@sequelize/core";
import process from "process";
import { Journal } from "@backend/entity";

export async function add(a: number, b: number): Promise<unknown> {
  const result = await Journal.build({ name: `${a} + ${b} = ${a + b}: ${new Date()}` }).save();
  return result.toJSON();
}

export async function init(options: SequelizeOptions) {
  const logger = createLogger({
    format: format.json(),
    transports: [new transports.Console()],
  });

  container.register<Logger>(Logger, {
    useValue: logger,
  });

  const sequelize = new Sequelize(process.env["WHITE_RABBIT_DATABASE_URL"] ?? "sqlite::memory:", {
    ...options,
    define: {
      underscored: true,
    },
    logging: (msg) => logger.info(msg, { class: "Sequelize" }),
    models: [Journal],
  });
  await sequelize.sync({ force: true });

  container.register<Sequelize>(Sequelize, {
    useValue: sequelize,
  });
}
