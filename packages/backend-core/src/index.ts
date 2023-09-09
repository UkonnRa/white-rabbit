import "reflect-metadata";
import "dotenv";

import { createLogger, format, transports, Logger } from "winston";
import { container } from "tsyringe";
import { Sequelize, type Options as SequelizeOptions } from "@sequelize/core";
import process from "process";
import {
  ACCOUNT_TYPES,
  AccountPO,
  AccountTagPO,
  EntryItemPO,
  EntryPO,
  EntryTagPO,
  JournalPO,
  JournalTagPO,
} from "@backend/entity";

export async function add(a: number, b: number): Promise<unknown> {
  const journal = await JournalPO.build({
    name: `${a} + ${b} = ${a + b}: ${new Date()}`,
    description: "Desc",
    unit: "Unit",
  }).save();
  await JournalTagPO.bulkCreate([
    { journalId: journal.id, tag: "tag 1" },
    { journalId: journal.id, tag: "tag 2" },
  ]);

  const accounts = await AccountPO.bulkCreate(
    ACCOUNT_TYPES.map((type) => ({
      journalId: journal.id,
      name: `${journal.name} - ${type}`,
      description: `Desc ${type}`,
      type,
      unit: `Unit ${type}`,
    })),
  );
  await AccountTagPO.bulkCreate(
    accounts.flatMap((account) => [
      { accountId: account.id, tag: "tag 1" },
      { accountId: account.id, tag: "tag 2" },
    ]),
  );

  const entries = await EntryPO.bulkCreate([
    {
      journalId: journal.id,
      name: `${journal.name} - Entry 1`,
      type: "RECORD",
      date: new Date("2022-01-01"),
    },
    {
      journalId: journal.id,
      name: `${journal.name} - Entry 2`,
      type: "RECORD",
      date: new Date("2023-01-01"),
    },
    {
      journalId: journal.id,
      name: `${journal.name} - Entry 3`,
      type: "RECORD",
    },
  ]);

  const entryItems = await EntryItemPO.bulkCreate(
    entries.flatMap((entry) =>
      accounts.map((account) => ({
        entryId: entry.id,
        accountId: account.id,
        amount: 1.2,
        price: Math.random() < 0.5 ? 3.4 : undefined,
      })),
    ),
  );

  return [
    journal.toJSON(),
    (await journal.getTags()).map((tag) => tag.toJSON()),
    accounts.map((model) => model.toJSON()),
    (await Promise.all(accounts.map((model) => model.getTags()))).flatMap((tags) =>
      tags.map((tag) => tag.toJSON()),
    ),
    entries.map((model) => model.toJSON()),
    entryItems.map((model) => model.toJSON()),
  ];
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
    models: [JournalPO, JournalTagPO, AccountPO, AccountTagPO, EntryPO, EntryTagPO, EntryItemPO],
  });
  await sequelize.sync({ force: true });

  container.register<Sequelize>(Sequelize, {
    useValue: sequelize,
  });
}
