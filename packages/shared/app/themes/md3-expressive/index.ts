import type { ThemeDefinition } from "../types";
import { rippleDirective } from "./ripple";
import { buttonRecipe } from "./recipes/button";
import { inputRecipe } from "./recipes/input";
import { textareaRecipe } from "./recipes/textarea";
import { selectRecipe } from "./recipes/select";
import { comboboxRecipe } from "./recipes/combobox";
import { cardRecipe } from "./recipes/card";
import { chipRecipe } from "./recipes/chip";
import { dialogRecipe } from "./recipes/dialog";
import { menuRecipe } from "./recipes/menu";
import { tooltipRecipe } from "./recipes/tooltip";
import { switchRecipe } from "./recipes/switch";
import { iconRecipe } from "./recipes/icon";
import { dataTableRecipe } from "./recipes/dataTable";
import { tagInputRecipe } from "./recipes/tagInput";

export const md3Expressive: ThemeDefinition = {
  name: "md3-expressive",
  recipes: {
    button: buttonRecipe,
    input: inputRecipe,
    textarea: textareaRecipe,
    select: selectRecipe,
    combobox: comboboxRecipe,
    card: cardRecipe,
    chip: chipRecipe,
    dialog: dialogRecipe,
    menu: menuRecipe,
    tooltip: tooltipRecipe,
    switch: switchRecipe,
    icon: iconRecipe,
    dataTable: dataTableRecipe,
    tagInput: tagInputRecipe,
  },
  directives: {
    ripple: rippleDirective,
  },
  stylesheets: ["./tokens.css", "./state-layer.css"],
};
