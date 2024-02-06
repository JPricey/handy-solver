import matplotlib.pyplot as plt
import os
from pathlib import Path
import csv
import matplotlib.colors as mcolors
import numpy as np

# DIR_PATH = os.path.dirname(os.path.realpath(__file__))
ROOT_PATH = Path(__file__).parent.parent
DATA_PATH = ROOT_PATH / "data" / "model_charts"

COLOURS = list(mcolors.BASE_COLORS.keys())


def field_transform(row, key, fn):
    row[key] = fn(row[key])


def row_transform(row):
    field_transform(row, "trial", int)
    field_transform(row, "iters", int)
    field_transform(row, "depth", int)
    field_transform(row, "duration_ms", float)


def extract_series_from_file(test_file, tag):
    all_datas = []
    with open(test_file, newline="") as csvfile:
        reader = csv.DictReader(csvfile)
        for row in reader:
            row_transform(row)
            row["tag"] = tag
            trial = row["trial"]
            while trial >= len(all_datas):
                all_datas.append([])
            all_datas[trial].append(row)
    return all_datas


def plot_datas(datas, opts={}):
    for line in datas:
        xs = [row["duration_ms"] for row in line]
        ys = [row["depth"] for row in line]
        line = plt.plot(xs, ys, **opts)


def plot_file(filename, color):
    data = extract_series_from_file(filename, "test")
    plot_datas(data, dict(color=color, linewidth=0.4))


# prefix = "Cursed.Spider-default-"
prefix = "Cursed.Spider-new-0-5-"


def main():
    files = list(DATA_PATH.glob(f"{prefix}*.csv"))
    print(files)

    for c, file in zip(COLOURS, files):
        plot_file(file, c)

    plt.legend(["a", "b"])
    plt.show()


def read_file(filename):
    all_datas = []
    with open(filename, newline="") as csvfile:
        reader = csv.DictReader(csvfile)
        for row in reader:
            row_transform(row)
            trial = row["trial"]
            while trial > len(all_datas):
                all_datas.append(
                    dict(trial=len(all_datas), iters=0, depth=0, duration_ms=10000)
                )
            all_datas.append(row)
    return all_datas


def load_durations():
    files = list(sorted(DATA_PATH.glob(f"Cursed.Spider*.csv")))
    percentiles = [25, 50, 75, 90, 95, 99, 99.9, 100]
    for filename in files:
        print(filename)
        rows = read_file(filename)
        x = [row["duration_ms"] for row in rows]
        x = np.array(x)
        # print(x)
        res = np.percentile(x, percentiles)
        print("Percentile Values: ", list(zip(percentiles, res)))

    plt.show()


# trial,duration_ms,iters,depth
DATA_COLUMNS = [
    "duration_ms",
    "iters",
    "depth",
]


def all_stats(data):
    res = dict()
    for column in DATA_COLUMNS:
        res[f"Avg {column}"] = avg_of_column(data, column)
    return str(res)


def avg_of_column(data, column_name):
    s = sum([row[column_name] for row in data])
    return float(s) / len(data)


def print_for_all(tagged_datas, fn):
    for thing in tagged_datas:
        print(thing[0])
        print(fn(thing[1]))


def stats_main():
    matchup = "Cursed.Spider"
    files = [
        f"{matchup}-weighted-0-10-1.csv",
        f"{matchup}-old-0-10-1.csv",
    ]
    files = [DATA_PATH / f for f in files]

    tagged_datas = [(file, read_file(file)) for file in files]

    print_for_all(tagged_datas, all_stats)


try:
    stats_main()
finally:
    plt.close()

# xs  = [2, 3, 4]
# ys  = [x*2 for x in xs]
