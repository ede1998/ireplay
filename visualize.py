import numpy as np
import matplotlib.pyplot as plt

# Read the sequence of zeros and ones from stdin
try:
    input_signal = input()
except EOFError as e:
    print(e)
    exit

# Convert the input string to an array of integers (0s and 1s)
signal = np.array([int(bit) for bit in input_signal])

# Generate time values corresponding to the length of the signal
sampling_rate = 4  # Sampling rate to visualize the signal clearly
duration = len(signal) / sampling_rate  # Duration of the signal based on input length
t = np.linspace(0, duration, len(signal), endpoint=False)

# Plotting the signal
plt.figure(figsize=(10, 4))

# Fill the area beneath the signal
plt.fill_between(t, signal, step='pre', color='lightblue', alpha=0.5)

# Plot the digital signal as steps
plt.step(t, signal, drawstyle='steps-post', color='black')

# Adding labels and title
plt.xlabel('Time [ms]')
plt.ylabel('Amplitude')
plt.grid(True)

# Show the plot
plt.tight_layout()
plt.show()
