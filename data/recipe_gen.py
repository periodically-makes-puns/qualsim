import csv


items = {}
with open("Item.csv", newline="") as f:
    lines = f.readlines() 
    lines = [lines[1]] + lines[4:] # discard leading key line and type
    csvf = csv.DictReader(lines)
    for row in csvf:
        if len(row["Name"]) > 0:
            items[int(row["#"])] = row["Name"]


rlt = {}
with open("RecipeLevelTable.csv", newline="") as f:
    lines = f.readlines() 
    lines = [lines[1]] + lines[4:] # discard leading key line and type
    csvf = csv.DictReader(lines)
    for row in csvf:
        rlt[int(row["#"])] = {
            "lvl": int(row["ClassJobLevel"]),
            "stars": int(row["Stars"]),
            "prog": int(row["Difficulty"]),
            "qual": int(row["Quality"]),
            "pdiv": int(row["ProgressDivider"]),
            "qdiv": int(row["QualityDivider"]),
            "pmod": int(row["ProgressModifier"]),
            "qmod": int(row["QualityModifier"]),
            "dur": int(row["Durability"])
        }

recipes = []
registered = set()
with open("Recipe.csv", newline="") as f:
    lines = f.readlines() 
    lines = [lines[1]] + lines[4:] # discard leading key line and type
    csvf = csv.DictReader(lines)
    for row in csvf:
        if not int(row[r"Item{Result}"]): continue
        rlvl = int(row["RecipeLevelTable"])
        idtup = (rlvl, int(row["DifficultyFactor"]), int(row["QualityFactor"]), int(row["DurabilityFactor"]), row["IsExpert"] == "True")
        if idtup in registered: continue
        else: registered.add(idtup)
        recipes.append({
            "name": items[int(row[r"Item{Result}"])],
            "rlvl": rlvl,
            "lvl": rlt[rlvl]["lvl"],
            "stars": rlt[rlvl]["stars"],
            "prog": int(row["DifficultyFactor"]) * rlt[rlvl]["prog"] // 100,
            "qual": int(row["QualityFactor"]) * rlt[rlvl]["qual"] // 100,
            "dur": int(row["DurabilityFactor"]) * rlt[rlvl]["dur"] // 100,
            "pdiv": rlt[rlvl]["pdiv"],
            "qdiv": rlt[rlvl]["qdiv"],
            "pmod": rlt[rlvl]["pmod"],
            "qmod": rlt[rlvl]["qmod"],
            "reqqual": int(row["RequiredQuality"]),
            "expert": row["IsExpert"] == "True"
        })

with open("recipes_filt.csv", "w", newline="") as f:
    csvf = csv.DictWriter(f, recipes[0].keys())
    csvf.writeheader()
    for recipe in recipes:
        csvf.writerow(recipe)
    

