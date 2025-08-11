struct Complex do
  let re = 0
  let im = 0

  func Complex i_re i_im do
    re = i_re
    im = i_im
  end

  func add other do
    re = re + other.re
    im = im + other.im
  end

  func sub other do
    re = re - other.re
    im = im - other.im
  end

  func mul other do
    let new_re = re * other.re - im * other.im
    let new_im = re * other.im - im * other.re
    re = new_re
    im = new_im
  end

  func div other do
    let denom = (other.re ^ 2) + (other.im ^ 2)
    let new_re = ((re * other.re) + (im * other.im)) / denom
    let new_im = ((im * other.re) - (re * other.im)) / denom
    re = new_re
    im = new_im
  end

  func as_string do
    return "" + re + " + " + im + "i"
  end

  func mag do
    return ((re ^ 2) + (im ^ 2)) ^ 0.5
  end

  func norm do
    let magnitude = mag()
    if magnitude == 0 do
      return new Complex(0, 0)
    else
      return new Complex(re / magnitude, im / magnitude)
    end
  end
end
