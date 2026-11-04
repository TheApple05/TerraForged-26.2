import java.util.*;
/** Genuine Java runtime Random/Collections observations; no native expected values. */
public final class NativeJavaRandomProbe {
    private static final long[] SEEDS={0L,1L,-1L,Long.MIN_VALUE,Long.MAX_VALUE,Integer.MIN_VALUE,Integer.MAX_VALUE,8675309L,8675309L+(1L<<48)};
    private static final int[] BOUNDS={1,2,3,50,100,1073741824,1073741825,Integer.MAX_VALUE};
    private static String bits(float f) {return String.format("%08x",Float.floatToRawIntBits(f));}
    public static void main(String[] args) {
        for(long seed:SEEDS) {
            Random r=new Random(seed);
            for(int step=0;step<256;step++) {
                StringJoiner fields=new StringJoiner(",");fields.add(Integer.toString(r.nextInt()));fields.add(Boolean.toString(r.nextBoolean()));fields.add(bits(r.nextFloat()));
                for(int bound:BOUNDS) fields.add(Integer.toString(r.nextInt(bound)));
                fields.add(Integer.toString(r.nextInt()));System.out.println("trace/"+seed+"/"+step+"\t"+fields);
            }
            for(int size:new int[]{0,1,2,5,32,257,4096}) {
                List<Integer> values=new ArrayList<>();for(int i=0;i<size;i++) values.add(i);
                Random shuffle=new Random(seed);Collections.shuffle(values,shuffle);
                System.out.println("shuffle/"+seed+"/"+size+"\t"+values.stream().map(Object::toString).collect(java.util.stream.Collectors.joining(",")));
                System.out.println("shuffle-next/"+seed+"/"+size+"\t"+shuffle.nextInt());
            }
            r=new Random(~seed);for(int i=0;i<32;i++) r.nextInt();r.setSeed(seed);
            for(int i=0;i<32;i++) System.out.println("reset/"+seed+"/"+i+"\t"+r.nextInt());
        }
    }
}
