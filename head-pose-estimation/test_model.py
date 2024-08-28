import torch
import torchvision.io
from torchvision.io import ImageReadMode
from PIL import Image

import utils.datasets
from model import DirectMHPInfer

if __name__ == '__main__':
    device = torch.device("cuda:0" if torch.cuda.is_available() else "cpu")
    model = DirectMHPInfer(weights='./weights/agora_m_best.pt', device=device)

    dataset = utils.datasets.LoadImages('img.jpg', 1280, int(model.model.stride.max()), auto=True)
    diter = iter(dataset)

    (p, img, im0, _) = next(diter)

    img = torch.from_numpy(img).to(device=device)
    img = img / 255.0

    if len(img.shape) == 3:
        img = img[None]

    y = model(img)

    print(y[0][0])

    torch.save(model.state_dict(), "../gesture-ease/weights.pt")
