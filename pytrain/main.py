import keras
import tensorflow as tf
from pathlib import Path
import jsonlines
from keras.models import Sequential
from keras.layers import Dense, Dropout, Activation, Flatten
from keras.layers import Convolution2D, MaxPooling2D
import numpy as np
from pprint import pprint
import json
import ctypes

rust_lib = ctypes.CDLL("../target/release/libhandy_c_lib.so")
rust_lib.next_pile_states.restype = ctypes.c_char_p
rust_lib.get_example.restype = ctypes.c_char_p
rust_lib.get_won_pile.restype = ctypes.c_char_p


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
    model.add(
        Dense(
            64,
            input_shape=(CARD_SIZE * PILE_SIZE,),
            kernel_initializer="RandomNormal",
            activation="relu",
        )
    )
    model.add(Dense(64, kernel_initializer="RandomNormal", activation="relu"))
    model.add(Dense(16, kernel_initializer="RandomNormal", activation="relu"))
    model.add(Dense(1, kernel_initializer="RandomNormal", activation="relu"))

    # print(model.summary())
    # for layer in model.layers:
    #     print(layer.get_output_at(0).get_shape().as_list())

    model.compile(
        loss="mean_squared_error",
        optimizer=keras.optimizers.AdamW(),
        metrics=["accuracy"],
    )

    return model


def pile_to_bits(pile):
    pile_input = [0.0] * CARD_SIZE * PILE_SIZE
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
    inputs = []
    outputs = []

    for obj in KNOWN_DATA:
        pile = obj["pile"]
        dist = obj["eval"]["Win"]

        inputs.append(pile_to_bits(pile))
        outputs.append(float(dist))

    model_predict = model.predict(inputs)
    score = 0
    num_inputs = len(inputs)
    for i in range(num_inputs):
        diff = outputs[i] - model_predict[i]
        score += (diff * diff) / num_inputs

    print("evaluation", score)


EVALUATION_RATE = 100
RANDOM_BATCH_SIZE = 1
WON_BATCH_SIZE = 1
TOTAL_BATCH_SIZE = RANDOM_BATCH_SIZE + WON_BATCH_SIZE

EPOCH_COUNT = 1

def avg(inputs):
    return sum(inputs) / len(inputs)


def evaluate_win_vs_not_win(model):
    inputs = []
    for _ in range(10):
        inputs.append(get_single_won_example())
    outputs = [x[0] for x in model.predict(inputs)]
    print("WINNING OUTPUTS:", avg(outputs), outputs)

    inputs = []
    for _ in range(10):
        inputs.append(get_single_not_won_example())
    outputs = [x[0] for x in model.predict(inputs)]
    print("NON-WINNING OUTPUTS:", avg(outputs), outputs)


def main(hero, monster):
    _init_card_map()
    _init_known_data(hero, monster)

    model = build_model()

    while True:
        for round_index in range(EVALUATION_RATE):
            print('round:', round_index)
            inputs = []
            outputs = []
            for b in range(RANDOM_BATCH_SIZE):
                # print(b)
                i, o = get_single_random_example(model)
                inputs.append(i)
                outputs.append(o)
            for b in range(WON_BATCH_SIZE):
                inputs.append(get_single_won_example())
                outputs.append(0.0)
            # pprint(outputs)
            model.fit(inputs, outputs, epochs=EPOCH_COUNT, verbose=0)
        evaluate_win_vs_not_win(model)
        # evaluate_model(model)


main("Cursed", "Demon")
