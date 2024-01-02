import os

os.environ["KERAS_BACKEND"] = "torch"

import keras
from pathlib import Path
import jsonlines
from keras.models import Sequential
from dataclasses import dataclass
from keras.layers import Dense, Dropout, Activation, Flatten, Input
from keras.layers import Convolution2D, MaxPooling2D, Conv1D
import numpy as np
from pprint import pprint
import json
import ctypes
import pdb
import torch
import random

rust_lib = ctypes.CDLL("../target/release/libhandy_c_lib.so")
rust_lib.next_pile_states.restype = ctypes.c_char_p
rust_lib.get_example.restype = ctypes.c_char_p
rust_lib.get_won_pile.restype = ctypes.c_char_p
rust_lib.get_deep_example.restype = ctypes.c_char_p


def next_pile_states(pile_string):
    input_bytes = pile_string.encode("utf-8")
    ptr = rust_lib.next_pile_states(input_bytes)
    result = ctypes.c_char_p(ptr).value.decode("utf-8")
    result = json.loads(result)
    pprint(result)
    # rust_lib.free_string(ptr)


def get_example():
    ptr = rust_lib.get_example()
    result = ctypes.c_char_p(ptr).value.decode("utf-8")
    result = json.loads(result)
    return result


def get_won_pile():
    ptr = rust_lib.get_won_pile()
    result = ctypes.c_char_p(ptr).value.decode("utf-8")
    result = json.loads(result)
    return result


def get_deep_example():
    ptr = rust_lib.get_deep_example()
    result = ctypes.c_char_p(ptr).value.decode("utf-8")
    result = json.loads(result)
    return result


DATA_DIR = Path("../data/training_data")


def matchup_training_data_path(hero, monster):
    return DATA_DIR.joinpath(f"{hero}.{monster}.jsonl")


# each card is a bitmap of 9 elements, and then a bitmap of 4 elements = 13 bits
# each example is 9 cards, and then the desired output

# shape should be (EXAMPLES, 9, 13) i think?? first dim is cards, next dim is card description
# convolution size would be 13?
# ok maybe let's ignore the convolution for now
# y_train should be (EXAMPLES,)

FACE_MAP = {
    "A": 0,
    "B": 1,
    "C": 2,
    "D": 3,
}

CARD_MAP = {}

CARD_SIZE = 13
PILE_SIZE = 9
CARD_ENCODING_SIZE = CARD_SIZE * PILE_SIZE

LOSE_GAME_REWARD = 100


def _init_card_map():
    assert CARD_MAP == {}
    example = get_example()
    pile = example["parent_pile"]

    card_nums = sorted([c[0] for c in pile])
    for i, c in enumerate(card_nums):
        CARD_MAP[c] = i


def build_model():
    model = Sequential()
    model.add(Input(shape=(PILE_SIZE, CARD_SIZE)))
    model.add(Conv1D(16, 1, 1, kernel_initializer="RandomNormal"))
    model.add(Flatten())
    model.add(
        Dense(
            32,
            kernel_initializer="RandomNormal",
            activation="relu",
        )
    )
    model.add(Dense(32, kernel_initializer="RandomNormal", activation="relu"))
    # model.add(Dense(32, kernel_initializer="RandomNormal", activation="relu"))
    model.add(Dense(1, kernel_initializer="RandomNormal", activation="relu"))

    model.summary()
    # for layer in model.layers:
    #     print(layer.get_output_at(0).get_shape().as_list())

    model.compile(
        loss="mean_squared_error",
        optimizer=keras.optimizers.AdamW(),
        metrics=["accuracy"],
    )

    return model


def pile_to_bits(pile):
    pile_input = torch.zeros(CARD_ENCODING_SIZE)
    for i, c in enumerate(pile):
        pile_input[CARD_SIZE * i + CARD_MAP[c[0]]] = 1.0
        pile_input[CARD_SIZE * i + PILE_SIZE + FACE_MAP[c[1]]] = 1.0
    return pile_input


def get_child_score(model, child_piles):
    child_bits = []
    for c in child_piles:
        if c["winner"] != "n":
            if c["winner"] == "h":
                return 0
        else:
            child_bits.append(pile_to_bits(c["pile"]))

    if len(child_bits) == 0:
        return None

    return min(x[0] for x in model.predict(child_bits, verbose=0))


def get_single_random_example(model):
    json_example = get_example()
    parent_pile_bits = pile_to_bits(json_example["parent_pile"])
    child_min_score = get_child_score(model, json_example["child_piles"])
    if child_min_score == None:
        return get_single_random_example(model)
    # print('cms', child_min_score)
    return (parent_pile_bits, child_min_score + 1)


def get_single_not_won_example():
    json_example = get_example()
    parent_pile_bits = pile_to_bits(json_example["parent_pile"])
    return parent_pile_bits


def get_single_won_example():
    json_example = get_won_pile()
    pile_bits = pile_to_bits(json_example)
    return pile_bits


KNOWN_DATA = []


def _init_known_data(hero, monster):
    with jsonlines.open(matchup_training_data_path(hero, monster)) as reader:
        for obj in reader:
            KNOWN_DATA.append(obj)


def evaluate_model(model):
    inputs = torch.zeros(len(KNOWN_DATA), CARD_ENCODING_SIZE)
    outputs = torch.zeros(len(KNOWN_DATA), 1)

    for i, obj in enumerate(KNOWN_DATA):
        pile = obj["pile"]
        dist = obj["eval"]["Win"]

        inputs[i] = pile_to_bits(pile)
        outputs[i] = float(dist)

    model_predict = model.predict(inputs)
    score = 0
    num_inputs = len(inputs)
    for i in range(num_inputs):
        diff = outputs[i] - model_predict[i]
        score += (diff * diff) / num_inputs

    print("evaluation", score)


EVALUATION_RATE = 10
RANDOM_BATCH_SIZE = 1
WON_BATCH_SIZE = 1
TOTAL_BATCH_SIZE = RANDOM_BATCH_SIZE + WON_BATCH_SIZE

EPOCH_COUNT = 1


@dataclass
class Example:
    depth: int
    winner: str
    weight: bool
    score: float
    children: list
    bits: list


def _pile_to_string(pile):
    return "".join("".join(str(e) for e in card) for card in pile)


def get_examples_from_tree(model):
    deep_example_tup = get_deep_example()
    deep_examples = deep_example_tup["examples"]
    max_depth = deep_example_tup["max_depth"]
    print(
        "depth:",
        max_depth,
        ". num states:",
        len(deep_examples),
        ". root pile:",
        _pile_to_string(deep_example_tup["root_pile"]),
    )
    examples = dict()

    for example in deep_examples:
        pile, data = example
        pile_string = _pile_to_string(pile)
        examples[pile_string] = Example(
            data["depth"],
            data["winner"],
            None,
            None,
            [_pile_to_string(c) for c in data["children"]],
            pile_to_bits(pile),
        )

    inputs = []
    weights = []
    outputs = []

    def _get_pile_score_and_weight(pile_string):
        # pdb.set_trace()
        example = examples[pile_string]
        if example.score is not None:
            return (example.score, example.weight)

        if example.winner == "b":
            return None

        if example.winner == "h":
            example.score = 0.0
            example.weight = 1.0
            return (example.score, example.weight)

        if example.depth == max_depth:
            example.weight = 0.0
            inp = torch.empty(1, CARD_ENCODING_SIZE)
            inp[0] = example.bits
            example.score = model.predict(inp, verbose=0)[0][0]
            # print('predict out', example.score)
            return (example.score, example.weight)

        res = None
        for child_pile_str in example.children:
            c_res = _get_pile_score_and_weight(child_pile_str)
            if c_res is not None:
                if res is None or c_res[0] < res[0]:
                    res = c_res
                elif c_res[0] == res[0] and c_res[1] > res[1]:
                    res = c_res

        if res is None:
            example.winner = "b"
            return None

        example.score = res[0] + 1
        example.weight = res[1] + 1
        return (example.score, example.weight)

    for pile_string, example in examples.items():
        sw = _get_pile_score_and_weight(pile_string)
        if sw is None:
            continue

        score, weight = sw

        if weight <= 0.0:
            continue

        inputs.append(example.bits)
        weights.append(weight)
        outputs.append(score)
        # print(pile_string, weight, score, example.depth)

    return inputs, weights, outputs


def avg(inputs):
    return sum(inputs) / len(inputs)


def evaluate_win_vs_not_win(model):
    EACH_SIZE = 10

    inputs = torch.zeros(EACH_SIZE, CARD_ENCODING_SIZE)
    for i in range(EACH_SIZE):
        inputs[i] = get_single_won_example()
    outputs = [x[0] for x in model.predict(inputs)]
    print("WINNING OUTPUTS:", avg(outputs), outputs)

    for i in range(EACH_SIZE):
        inputs[i] = get_single_not_won_example()
    outputs = [x[0] for x in model.predict(inputs)]
    print("NON-WINNING OUTPUTS:", avg(outputs), outputs)


def main(hero, monster):
    _init_card_map()
    _init_known_data(hero, monster)

    model = build_model()

    # pdb.set_trace()

    while True:
        for round_index in range(EVALUATION_RATE):
            print("round:", round_index)
            inputs, weights, outputs = get_examples_from_tree(model)
            len_inputs = len(inputs)
            print("num examples:", len_inputs)
            if len_inputs == 0:
                continue

            num_won_states = len_inputs // 10
            print("adding won examples:", num_won_states)
            for _ in range(num_won_states):
                inputs.append(get_single_won_example())
                weights.append(1.0)
                outputs.append(0.0)

            full_len_inputs = len(inputs)
            indexes = [x for x in range(full_len_inputs)]
            random.shuffle(indexes)

            for idx in indexes:
                inp = torch.empty(1, CARD_ENCODING_SIZE)
                inp[0] = inputs[idx]

                out = torch.empty(1, 1)
                out[0][0] = outputs[idx]

                # print('fitting', inp, out)
                model.fit(inp, out, verbose=0)

        evaluate_win_vs_not_win(model)
        evaluate_model(model)


main("Cursed", "Demon")
