# import keras
# import tensorflow as tf

from pathlib import Path
import jsonlines
from pprint import pprint
from keras.models import Sequential
from keras.layers import Dense, Dropout, Activation, Flatten
from keras.layers import Convolution2D, MaxPooling2D
# from keras.utils import np_utils
import numpy as np
np.random.seed(123)

DATA_DIR = Path('../data/training_data')
print(DATA_DIR)

def matchup_training_data_path(hero, monster):
    return DATA_DIR.joinpath(f'{hero}.{monster}.jsonl')

# each card is a bitmap of 9 elements, and then a bitmap of 4 elements = 13 bits
# each example is 9 cards, and then the desired output

# shape should be (EXAMPLES, 9, 13) i think?? first dim is cards, next dim is card description
# convolution size would be 13?
# ok maybe let's ignore the convolution for now
# y_train should be (EXAMPLES,)

def main(hero, monster):
    cur_training_path = matchup_training_data_path(hero, monster)
    print(cur_training_path)

    face_map = {
            'A': 0,
            'B': 1,
            'C': 2,
            'D': 3,
    }
    card_map = {}

    inputs = []
    outputs = []

    with jsonlines.open(cur_training_path) as reader:
        first = True
        for obj in reader:
            pile = obj['pile']
            dist = obj['eval']['Win']

            if first:
                first = False
                pile = obj['pile']
                card_nums = sorted([c[0] for c in pile])
                for i, c in enumerate(card_nums):
                    card_map[c] = i

            pile_input = []
            for c in pile:
                cur_input = [0.0] * 13
                cur_input[card_map[c[0]]] = 1.0
                cur_input[face_map[c[1]] + 9] = 1.0
                pile_input.append(cur_input)
            inputs.append(pile_input)
            outputs.append(float(dist))

    x_train = np.array(inputs[1000:])
    y_train = np.array(outputs[1000:])

    x_test = np.array(inputs[:1000])
    y_test = np.array(outputs[:1000])
    print(x_train.shape, y_train.shape)
    print(x_test.shape, y_test.shape)

    model = Sequential()
    model.add(Dense(8, input_shape=(9, 13,), activation='relu'))
    model.add(Flatten())
    model.add(Dense(16, activation='relu'))
    model.add(Dense(1, activation='relu'))

    print(model.summary())
    for layer in model.layers:
        print(layer.get_output_at(0).get_shape().as_list())

    model.compile(loss='mean_squared_error', optimizer='adam', metrics=['accuracy'])
    model.fit(x_train, y_train, epochs=200, verbose=1)
    evalres, accuracy = model.evaluate(x_train, y_train)

    def evaluate_model(xs, ys):
        score = 0.0
        predict = model.predict(xs)
        for i, p in enumerate(predict):
            p = p[0]
            diff = p - ys[i]
            score += diff*diff
            # print(i, 'p:', p, 'diff:', diff)

        score /= len(predict)
        print('final score', score)

    evaluate_model(x_train, y_train)
    evaluate_model(x_test, y_test)


# main('Pyro', 'Demon')
main('Warrior', 'Spider')
