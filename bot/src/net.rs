use rand::Rng;


pub struct NetImage {
    pub data : Vec<f32>,
    pub w : usize,
    pub h : usize,
    pub c : usize,
    pub y_stride : usize,
    pub x_stride : usize
}

impl NetImage {
    pub fn new(w : usize, h : usize, c : usize) -> Self {
        NetImage {
            w,
            h,
            c,
            x_stride : c,
            y_stride : w * c,
            data : vec![0.0; w * h * c]
        }
    }

    #[inline(always)]
    pub fn get(&self, x : usize, y : usize, c : usize) -> f32 {
        self.data[y * self.y_stride + x * self.x_stride + c]
    }

    #[inline(always)]
    pub fn get_mut(&mut self, x : usize, y : usize, c : usize) -> &mut f32 {
        &mut self.data[y * self.y_stride + x * self.x_stride + c]
    }

    pub fn get_channel_slice(&self, x : usize, y : usize) -> &[f32] {
        let start_idx = y * self.y_stride + x * self.x_stride;
        let end_idx = start_idx + self.c;
        &self.data[start_idx..end_idx]
    }

    pub fn get_channel_slice_mut(&mut self, x : usize, y : usize) -> &mut [f32] {
        let start_idx = y * self.y_stride + x * self.x_stride;
        let end_idx = start_idx + self.c;
        &mut self.data[start_idx..end_idx]
    }
}

pub trait Layer {
    fn process(&mut self, inp : &NetImage, dst : &mut NetImage);
    fn allocate_output(&mut self, inp : &NetImage) -> NetImage;
}

pub struct Node {
    pub layer : Box<dyn Layer>,
    pub cache : Option<NetImage>
}

impl Node {
    pub fn new<T : Layer + 'static>(layer : T) -> Node {
        Node {
            layer : Box::new(layer),
            cache : None
        }
    }
}

impl Node {
    fn process(&mut self, inp : &NetImage) {
        self.layer.process(inp, self.cache.as_mut().unwrap());
    }
}

pub struct PReLU {
    pub k : Vec<f32>
}

impl PReLU {
    pub fn new(c : usize) -> Self {
        let mut rnd = rand::thread_rng();
        let mut k = vec![0.0; c];
        for idx in 0..c {
            k[idx] = rnd.gen_range(-1.0..=1.0);
        }
        Self {
            k
        }
    }
}

impl Layer for PReLU {
    fn process(&mut self, inp: &NetImage, dst: &mut NetImage) {
        for y in 0..inp.h {
            for x in 0..inp.w {
                let in_c = inp.get_channel_slice(x, y);
                let mut out_c = dst.get_channel_slice_mut(x, y);
                for c in 0..self.k.len() {
                    out_c[c] = if in_c[c] >= 0.0 {
                        in_c[c]
                    } else {
                        in_c[c] * self.k[c]
                    };
                }
            }
        }
    }

    fn allocate_output(&mut self, inp: &NetImage) -> NetImage {
        NetImage::new(inp.w, inp.h, self.k.len())
    }
}

pub struct SimpleNetwork {
    pub nodes : Vec<Node>
}

impl SimpleNetwork {

    pub fn extend(&mut self, other : SimpleNetwork) {
        self.nodes.extend(other.nodes);
    }

    pub fn push(&mut self, node : Node) {
        self.nodes.push(node);
    }

    pub fn central_conv2d(w : usize, h : usize, in_c : usize, out_c : usize) -> Self {
        let pad = Node::new(Padding::new(w / 2, h / 2));
        let conv = Node::new(Conv2d::new(w, h, in_c, out_c));
        SimpleNetwork {
            nodes : vec![pad, conv]
        }
    }

    pub fn simple_maker(
        conv_size : usize,
        in_c : usize,
        inner_c : usize,
        out_c : usize,
        layers : usize) -> Self {

        let mut res = SimpleNetwork {
            nodes : vec![]
        };

        res.extend(SimpleNetwork::central_conv2d(conv_size, conv_size, in_c, inner_c));
        res.push(Node::new(PReLU::new(inner_c)));
        for idx in 0..layers {
            res.extend(SimpleNetwork::central_conv2d(conv_size, conv_size, inner_c, inner_c));
            res.push(Node::new(PReLU::new(inner_c)));
        }

        res.extend(SimpleNetwork::central_conv2d(conv_size, conv_size, inner_c, out_c));

        res
    }
}

impl Layer for SimpleNetwork {
    fn process(&mut self, inp: &NetImage, dst: &mut NetImage) {

        self.nodes[0].process(inp);
        for idx in 1..(self.nodes.len() - 1) {
            let n_inp = self.nodes[idx - 1].cache.take().unwrap();
            self.nodes[idx].process(&n_inp);
            self.nodes[idx - 1].cache = Some(n_inp);
        }

        let idx = self.nodes.len() - 1;
        let n_inp = self.nodes[idx - 1].cache.take().unwrap();
        self.nodes[idx].layer.process(&n_inp, dst);
        self.nodes[idx - 1].cache = Some(n_inp);
    }

    fn allocate_output(&mut self, inp: &NetImage) -> NetImage {
        let mut cur_img = inp;
        for n in &mut self.nodes {
            let out = n.layer.allocate_output(cur_img);
            n.cache =Some(out);
            cur_img = n.cache.as_ref().unwrap();
        }
        self.nodes.last_mut().unwrap().cache.take().unwrap()
    }
}

pub struct Padding {
    pub pad_w : usize,
    pub pad_h : usize
}

impl Padding {
    pub fn new(pad_w : usize, pad_h : usize) -> Padding {
        Padding {
            pad_w,
            pad_h
        }
    }
}

impl Layer for Padding {
    fn process(&mut self, inp: &NetImage, dst: &mut NetImage) {
        for y in 0..inp.h {
            for x in 0..inp.w {
                for c in 0..inp.c {
                    *dst.get_mut(x + self.pad_w, y + self.pad_h, c) = inp.get(x, y, c);
                }
            }
        }
    }

    fn allocate_output(&mut self, inp: &NetImage) -> NetImage {
        NetImage::new(inp.w + self.pad_w * 2, inp.h + self.pad_h * 2, inp.c)
    }
}

pub struct Conv2d {
    pub weights : Vec<f32>,
    pub w : usize,
    pub h : usize,
    pub in_c : usize,
    pub out_c : usize
}

impl Conv2d {
    fn new(w : usize, h : usize, in_c : usize, out_c : usize) -> Self {
        let mut rnd = rand::thread_rng();
        let mut weights = vec![0.0; w * h * in_c * out_c];
        for idx in 0..weights.len() {
            weights[idx] = rnd.gen_range(-1.0..=1.0);
        }

        Self {
            weights,
            w,
            h,
            in_c,
            out_c
        }
    }
}

impl Layer for Conv2d {
    fn process(&mut self, inp: &NetImage, dst: &mut NetImage) {
        for y in 0..dst.h {
            for x in 0..dst.w {
                for c in 0..self.out_c {
                    let mut sum = 0.0_f32;
                    for dy in 0..self.h {
                        for dx in 0..self.w {
                            let inp_vals = inp.get_channel_slice(x + dx, y + dy);
                            let w_idx = ((c * self.h + dy) * self.w + dx) * self.in_c;
                            for in_c in 0..self.in_c {
                                sum += inp_vals[in_c] * self.weights[w_idx + in_c];
                            }
                        }
                    }
                    *dst.get_mut(x, y, c) = sum;
                }
            }
        }
    }

    fn allocate_output(&mut self, inp: &NetImage) -> NetImage {
        NetImage::new(inp.w - self.w + 1, inp.h - self.h + 1, self.out_c)
    }
}